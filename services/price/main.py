import os

pth = '/etc/secret-volume/pubsub_keys.json'
if os.path.exists(pth):
    os.environ['GOOGLE_APPLICATION_CREDENTIALS'] = pth

from google.cloud import pubsub_v1
import base64
import mysql.connector
from time import sleep
import json


db_config = json.load(open('/etc/secret-volume/db_config.json'))


def execute_sql(query, params=None, mode='select', db=None):
    if db not in ('price', 'owner'):
        raise ValueError('db must be one of `price` or `owner`')
    with mysql.connector.connect(**db_config, database=db) as conn:
        cursor = conn.cursor()
        cursor.execute(query, params)
        if mode == 'select':
            return conn.fetchall()
        elif mode == 'commit':
            conn.commit()


def dict_update(d, k, v):
    d[k] = v
    return d


def callback(message):
    data = json.loads(bytes.decode(message.data))
    print('Data:', data)

    table = data['asset']

    if 'events' not in data:
        print(f'WARNING: Message does not contain any events ---- {data}')
        message.ack()
        return

    events = data['events']

    # must be 3 events for a fill to have occured (1 open, 2 fills (bid / ask))
    if len(events) >= 3 and 'Filled' in events[-2]:
        fill_events = list(map(lambda x: x['Filled'], filter(lambda x: 'Filled' in x, events)))

        vol = int(fill_events[-1]['size'])
        ts = int(fill_events[0]['timestamp'])
        price = sum(map(lambda x: float(x['price']) * float(x['size']), fill_events)) / float(vol)

        query = """
        INSERT INTO {}_price
            (market_time, open, low, high, close, volume)
        VALUES
            (%s, %s, %s, %s, %s, %s)
        ON DUPLICATE KEY UPDATE
            low = min(low, VALUES(low)),
            high = max(high, VALUES(high)),
            close = VALUES(close),
            volume = volume + VALUES(volume);
        """.format(table.lower())

        execute_sql(query, params=(ts, price, price, price, price, vol), mode='commit', db='price')

    message.ack()


if __name__ == '__main__':
    with pubsub_v1.SubscriberClient() as subscriber:
        sub_path = subscriber.subscription_path('project-steelieman', 'price-updates-sub')
        future = subscriber.subscribe(sub_path, callback)
        
        future.result()
