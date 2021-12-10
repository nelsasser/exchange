import uuid
from uuid import uuid1
import json
import os

pth = '/etc/secret-volume/pubsub_keys.json'
if os.path.exists(pth):
    os.environ['GOOGLE_APPLICATION_CREDENTIALS'] = pth

VALID_ASSETS = json.loads(os.environ['VALID_ASSETS'])

from flask import Flask, Response, request
from google.cloud import pubsub_v1
import mysql.connector

app = Flask(__name__)

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


def _parse_owner(data, errs):
    owner = None

    if 'owner' not in data:
        errs.append('Expected field `owner` not found.')
    else:
        owner = data['owner']

    if not isinstance(owner, str):
        errs.append('Expected field `owner` for be of type str')
    else:
        try:
            uuid.UUID(owner, version=1)
        except ValueError:
            errs.append('Field `owner` must be a valid RFC_4122 compliant UUID. Try GET */uuid to claim a valid id.')

    return owner


def _parse_asset(data, errs):
    asset = None

    if 'asset' not in data:
        errs.append('Expected field `asset` not found.')
    else:
        asset = data['asset']

    if not isinstance(asset, str):
        errs.append('Expected field `asset` for be of type str')
    else:
        if asset not in VALID_ASSETS:
            errs.append(f'Field `asset` must be one of {VALID_ASSETS}')

    return asset


def _parse_direction(data, errs):
    direction = None

    if 'direction' not in data:
        errs.append('Expected field `direction` not found.')
    else:
        direction = data['direction']

    if not isinstance(direction, str):
        errs.append('Expected field `direction` for be of type str')
    else:
        if direction not in ('Bid', 'Ask'):
            errs.append(f"Field `direction` must be one of ['Bid', 'Ask']")

    return direction


def _parse_price(data, errs):
    price = None

    if 'price' not in data:
        errs.append('Expected field `price` not found.')
    else:
        price = data['price']

    if type(price) not in (int, float):
        errs.append('Expected field `price` for be of type int or float')
    else:
        if price < 0:
            errs.append('Expected field `price` to be >= 0.0')

    return price


def _parse_size(data, errs):
    size = None

    if 'size' not in data:
        errs.append('Expected field `size` not found.')
    else:
        size = data['size']

    if not isinstance(size, int):
        errs.append('Expected field `size` for be of type int')
    else:
        if size < 0:
            errs.append('Expected field `size` to be >= 0')

    return size


def _parse_order(data, errs):
    order = None

    if 'order' not in data:
        errs.append('Expected field `order` not found.')
    else:
        order = data['order']

    if not isinstance(order, str):
        errs.append('Expected field `order` for be of type str')
    else:
        try:
            uuid.UUID(order, version=1)
        except ValueError:
            errs.append('Field `order` must be a valid RFC_4122 compliant UUID.')

    return order


def _parse_timestamp(data, errs, name=None):
    ts = None

    if name is None or not isinstance(name, str):
        raise ValueError('name must be a string')

    if name not in data:
        errs.append(f'Expected field `{name}` not found.')
    else:
        try:
            ts = int(data[name])
        except:
            errs.append(f'Could not convert timestamp {name} to int')

    return ts


@app.route('/')
def hello_world():
    return 'Hello world!'


@app.route('/uuid', methods=['GET'])
def trader_id():
    return {'uuid': str(uuid1())}


@app.route('/submit', methods=['POST'])
def submit_order():
    errs = []
    data = request.get_json()

    owner = _parse_owner(data, errs)
    asset = _parse_asset(data, errs)
    direction = _parse_direction(data, errs)
    price = _parse_price(data, errs)
    size = _parse_size(data, errs)

    if len(errs):
        # return errors
        return Response(json.dumps({'status': 'error', 'errors': errs}), status=400, mimetype='application/json')
    else:
        publisher = pubsub_v1.PublisherClient()
        topic = f'projects/{os.getenv("GOOGLE_CLOUD_PROJECT") or "project-steelieman"}/topics/{asset}'

        order = {
            'Open': {
                'owner': owner,
                'price': str(price),
                'size': str(size),
                'direction': direction,
                'timestamp': 0,
                'uuid': None,
            }
        }

        res = publisher.publish(topic, json.dumps(order).encode()).result()

        if res:
            return Response(json.dumps({'status': 'success', 'results': res, 'errors': errs}), status=200, mimetype='application/json')
        else:
            return Response(json.dumps({'status': 'error', 'errors': ['No results from order']}), status=500, mimetype='application/json')


@app.route('/cancel', methods=['POST'])
def cancel_order():
    errs = []
    data = request.get_json()

    owner = _parse_owner(data, errs)
    asset = _parse_asset(data, errs)
    order = _parse_order(data, errs)

    if len(errs):
        # return errors
        return Response(json.dumps({'status': 'error', 'errors': errs}), status=400, mimetype='application/json')
    else:
        publisher = pubsub_v1.PublisherClient()
        topic = f'projects/{os.getenv("GOOGLE_CLOUD_PROJECT") or "project-steelieman"}/topics/{asset}'

        cancel = {
            'Cancel': {
                'owner': owner,
                'id': order,
                'timestamp': 0,
            }
        }

        res = publisher.publish(topic, json.dumps(cancel).encode()).result()

        if res:
            return Response(json.dumps({'status': 'success', 'results': res, 'errors': errs}), status=200, mimetype='application/json')
        else:
            return Response(json.dumps({'status': 'error', 'errors': ['No results from cancellation']}), status=500, mimetype='application/json')


@app.route('/orders', methods=['POST'])
def orders():
    errs = []
    data = request.get_json()

    owner = _parse_owner(data, errs)
    asset = _parse_asset(data, errs)

    query = """
        SELECT
            *
        FROM
            accounts
        WHERE
            owner = %s AND asset = %s; 
    """

    res = None
    try:
        res = execute_sql(query, params=(owner.int, asset), mode='select', db='owner')
    except Exception as e:
        errs.append(str(e))

    if len(errs):
        # return errors
        return Response(json.dumps({'status': 'error', 'errors': errs, 'orders': res}), status=400, mimetype='application/json')
    else:
        return Response(json.dumps({'status': 'success', 'errors': errs, 'orders': res}), status=200, mimetype='application/json')


@app.route('/price', methods=['POST'])
def price():
    res = None
    errs = []
    data = request.get_json()

    asset = _parse_asset(data, errs)
    start_time = _parse_timestamp(data, errs, name='start_time')
    end_time = _parse_timestamp(data, errs, name='end_time')

    if start_time > end_time:
        errs.append('start_time must be leq end_time.')
    else:
        query = """
            SELECT
                *
            FROM
                {}_price
            WHERE
                market_time >= %s AND market_times <= %s;
        """.format(asset)

        try:
            res = execute_sql(query, params=(start_time, end_time), mode='select', db='price')
        except Exception as e:
            errs.append(str(e))

    if len(errs):
        # return errors
        return Response(json.dumps({'status': 'error', 'errors': errs, 'orders': res}), status=400, mimetype='application/json')
    else:
        return Response(json.dumps({'status': 'success', 'errors': errs, 'orders': res}), status=200, mimetype='application/json')


# start flask app
app.run(host="0.0.0.0", port=5000, debug=False)
