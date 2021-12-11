import os

pth = '/etc/secret-volume/pubsub_keys.json'
if os.path.exists(pth):
    os.environ['GOOGLE_APPLICATION_CREDENTIALS'] = pth
else:
    os.environ['GOOGLE_APPLICATION_CREDENTIALS'] = 'C:\\Users\\elsan\\Documents\\exchange\\pubsub_keys.json'

from google.cloud import pubsub_v1
import base64
import mysql.connector
from time import sleep
import json
import uuid

try:
    db_config = json.load(open('/etc/secret-volume/db_config.json'))
except:
    db_config = json.load(open('C:\\Users\\elsan\\Documents\\exchange\\db_config.json'))


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


def callback(message):
    data = json.loads(bytes.decode(message.data))
    print('Data:', data)

    asset = data['asset']

    if 'events' not in data:
        print(f'WARNING: Message does not contain any events ---- {data}')
        message.ack()
        return

    # filter out bounce events
    for event in filter(lambda x: 'Bounce' not in x, data['events']):
        status, event = list(event.items())[0]

        owner = uuid.UUID(event['owner']).int
        order = uuid.UUID(event['id']).int

        price = float(event['price']) if 'price' in event else 1.0
        size = int(event['size']) if 'size' in event else 1
        direction = event.get('direction', 'TEMP') # can be None if it is a filled or canceled eveny
        parent = uuid.UUID(event['parent']).int if event['parent'] else None

        query = """
            INSERT INTO accounts
                (owner_id, order_id, parent_id, asset, price, size, direction, status, timestamp)
            VALUES
                (%s, %s, %s, %s, %s, %s, %s, %s, DEFAULT)
            ON DUPLICATE KEY UPDATE
                price = price * VALUES(price),
                size = size * VALUES(size),
                direction = IF(direction = 'TEMP', VALUES(direction), direction),
                status = IF(status = 'Opened', VALUES(status), status), 
                timestamp = NOW();
        """

        execute_sql(query, params=(owner, order, parent, asset, price, size, direction, status), mode='commit', db='owner')

    message.ack()


if __name__ == '__main__':
    with pubsub_v1.SubscriberClient() as subscriber:
        sub_path = subscriber.subscription_path('project-steelieman', 'account-updates-sub')
        future = subscriber.subscribe(sub_path, callback)

        future.result()
