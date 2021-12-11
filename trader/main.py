import uuid
import requests
import random
import time

from multiprocessing import Pool


def run_trader(trader):
    open_trades = False

    owner = requests.get(f'http://{trader["endpoint"]}/uuid').json()['uuid']

    print(f'Trader #{trader["num"]} == {owner}')

    while True:
        if random.random() < 0.75 or not open_trades:
            # submit 5 trades
            for i in range(5):
                direction = 'Bid' if random.random() < 0.5 else 'Ask'
                price = round(max(0.0, random.normalvariate(trader["mean"], trader["var"]), 2))
                size = int((random.uniform(50, 100) // 10) * 10)

                print(f'Trader #{trader["num"]} submitting {size} @ {price}')

                res = requests.post(f'http://{trader["endpoint"]}/submit', json={
                    'owner': owner,
                    'asset': trader["asset"],
                    'direction': direction,
                    'price': price,
                    'size': size
                })

                if res.status_code != 200:
                    print('SUBMIT ERROR')
                    print(res.json()['errs'])
                    print()

            open_trades = True
        else:

            print(f'Trader #{trader["num"]} cancelling open orders.')

            # cancel all open trades
            # first get the orders to cancel
            data = {
                'owner': owner,
                'asset': trader["asset"],
            }
            orders = requests.post(f'http://{trader["endpoint"]}/orders', json=data).json()

            if 'orders' not in orders or orders['orders'] is None:
                if len(orders['errs']):
                    print('ERROR')
                    print(orders['errs'])
                    print('')
                return

            open_order_ids = map(lambda x: str(uuid.UUID(x[1])), filter(lambda x: x[5] == 'Opened', orders['orders']))

            for order_id in open_order_ids:
                data = {
                    'owner': owner,
                    'id': order_id,
                    'asset': trader["asset"],
                }

                res = requests.post(f'http://{trader["endpoint"]}/cancel', json=data)

                if res.status_code != 200:
                    print('ERROR')
                    print(res.json()['errs'])
                    print()
            open_trades = False

        time.sleep(trader["loop_delay"])


if __name__ == '__main__':
    url = "34.68.113.9"
    name = 'AAPL'
    mean = 100
    var = 5
    loop_delay = 0.1

    num_traders = 10

    traders = [{
        'num': i,
        'endpoint': url,
        'asset': name,
        'mean': mean,
        'var': var,
        'loop_delay': loop_delay
    } for i in range(num_traders)]

    with Pool(num_traders) as pool:
        pool.map(run_trader, traders, chunksize=1)
