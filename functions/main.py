import base64
import json
import math

from google.cloud import pubsub_v1

# TODO:
#  1) write owner account handler code (pubsub 2 pubsub => container writing updates to table)
#  2) write price update code (CHANGE TO SAME AS OWNER FOR SIMPLICITY!!! container writing updates to table)
#  3) Default db tables (accounts)
#  4) Default pubsub topics (price-updates, account-updates)
#  5) finish exchange deployment
#       - create new db tables for asset price
#       - create new cloud function for price extraction
#  6) remaining frontend apis to query database (account query, current price query, historical price query)

def event_price_handler(event, context):
    message = json.loads(base64.b64decode(event['data']).decode('utf-8'))

    if len(message['events']) > 1 and 'Filled' in message['events'][-2]:
        fill = message['events'][-2]['Filled']
        ret = {
            'asset': message['asset'],
            'volume': int(fill['size']),
            'price': float(fill['price']),
            'timestamp': math.ceil(float(fill['timestamp']) / (1000 * 60)) * 1000 * 60 # round to next minute, timestamp in milliseconds
        }

        publisher = pubsub_v1.PublisherClient()
        publisher.publish(f'projects/project-steelieman/topics/price-updates', json.dumps(ret).encode()).result()