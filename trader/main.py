import os
import uuid
import requests
import random
import time

url = os.getenv('REST') or '0.0.0.0:5000'


def submit_trade(asset, price_mean, price_var, trader_id, direction):
    res = requests.post(f'http://{url}/submit', json={
        'owner': trader_id,
        'asset': asset,
        'direction': direction,
        'price': round(max(0.0, random.normalvariate(price_mean, price_var), 2)),
        'size': int((random.uniform(50, 100) // 10) * 10)
    })

    if res.status_code != 200:
        print('SUBMIT ERROR')
        print(r.json()['errs'])
        print()


def cancel_orders(asset, trader_id):
    # first get the orders to cancel
    data = {
        'owner': trader_id,
        'asset': asset,
    }
    orders = requests.post(f'http://{url}/orders', json=data).json()

    if 'orders' not in orders:
        print('ERROR')
        print(orders['errs'])
        print('')
        return

    open_order_ids = map(lambda x: str(uuid.UUID(x[1])), filter(lambda x: x[5] == 'Opened', orders['orders']))

    for order_id in open_order_ids:
        data = {
            'owner': trader_id,
            'id': order_id,
            'asset': asset,
        }

        res = requests.post(f'http://{url}/cancel', json=data)

        if res.status_code != 200:
            print('ERROR')
            print(res.json()['errs'])
            print()


if __name__ == '__main__':
    name = 'AAPL'
    mean = 100
    var = 5

    r = requests.get(f'http://{url}/uuid')
    assert r.status_code == 200

    tid = r.json()['uuid']

    open_trades = False

    while True:
        if random.random() < 0.75 or not open_trades:
            # submit 5 trades
            for i in range(5):
                d = 'Bid' if random.random() < 0.5 else 'Ask'
                submit_trade(name, mean, var, tid, d)
            open_trades = True
        else:
            # cancel all open trades
            cancel_orders(name, tid)
            open_trades = False

        time.sleep(0.01)
