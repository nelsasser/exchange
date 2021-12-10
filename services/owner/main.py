import os

pth = '/etc/secret-volume/pubsub_keys.json'
if os.path.exists(pth):
    os.environ['GOOGLE_APPLICATION_CREDENTIALS'] = pth

from google.cloud import pubsub_v1
import base64
import mysql.connector
from time import sleep
import json
import uuid


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


def callback(message):
    data = json.loads(base64.b64decode(message.data).decode('utf-8'))
    print('Data:', data)

    asset = data['asset']

    # filter out bounce events
    for event in filter(lambda x: 'Bounce' not in x, data['events']):
        status, event = list(event.items())[0]

        owner = uuid.UUID(event['owner']).int
        order = uuid.UUID(event['id']).int

        if status == 'Opened':
            # insert new row for the order
            price = float(event['price'])
            size = int(event['size'])
            direction = event['direction']
            parent = uuid.UUID(event['parent']) if event['parent'] else None

            query = """
                INSERT INTO accounts
                    (owner_id, order_id, parent_id, asset, price, size, direction, status, timestamp)
                VALUES
                    (%s, %s, %s, %s, %s, %s, %s, %s, DEFAULT)
            """

            execute_sql(query, params=(owner, order, parent, asset, price, size, direction, status), mode='commit', db='owner')
        else:
            # update the row of the order with the new status (canceled / filled)
            query = """
                UPDATE 
                    accounts
                SET
                    status = %s,
                    timestamp = NOW()
                WHERE 
                    owner_id = %s AND 
                    order_id = %s
            """

            execute_sql(query, params=(status, owner, order), mode='commit', db='owner')

    message.ack()


if __name__ == '__main__':
    subscription = f'projects/project-steelieman/topics/account-updates-sub'

    print('Listening on', subscription)

    with pubsub_v1.SubscriberClient() as subscriber:
        future = subscriber.subscribe(subscription, callback)

        future.result()


    print('AAAAAHHH!')
