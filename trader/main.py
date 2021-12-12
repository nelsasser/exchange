import uuid
import requests
import random
import time
import math

from multiprocessing import Pool


def run_trader(trader):
    open_trades = False

    owner = requests.get(f'http://{trader["endpoint"]}/uuid').json()['uuid']

    print(f'Trader #{trader["num"]} == {owner}', flush=True)

    while True:
        if random.random() < 0.75 or not open_trades:
            # submit trade
            direction = 'Bid' if random.random() < 0.5 else 'Ask'
            s = math.sin(((int(time.time() * 1e6) % 60) / 60) * 2 * math.pi) * 10 # mean should vary to +/- $10 every minute
            price = round(max(0.0, random.normalvariate(trader[f"{direction} mean"] + s, trader["var"])), 2)
            size = int((random.uniform(50, 100) // 10) * 10)

            print(f'Trader #{trader["num"]} submitting {direction} {size} @ {price}', flush=True)

            res = requests.post(f'http://{trader["endpoint"]}/submit', json={
                'owner': owner,
                'asset': trader["asset"],
                'direction': direction,
                'price': price,
                'size': size
            })

            if res.status_code != 200:
                print(f'Trader #{trader["num"]} SUBMIT ERROR\n{res.json()["errors"]}', flush=True)

            open_trades = True
        else:

            print(f'Trader #{trader["num"]} cancelling open orders.', flush=True)

            # cancel all open trades
            # first get the orders to cancel
            data = {
                'owner': owner,
                'asset': trader["asset"],
            }

            r = None
            try:
                r = requests.post(f'http://{trader["endpoint"]}/orders', json=data)
                orders = r.json()
            except:
                print(f'Trader #{trader["num"]} ORDERS ERROR\n{r.text}\n', flush=True)
                continue

            if 'orders' not in orders or orders['orders'] is None:
                if len(orders['errors']):
                    print(f'Trader #{trader["num"]} ORDERS ERROR\n{orders["errors"]}\n', flush=True)
                continue

            open_order_ids = map(lambda x: str(uuid.UUID(x[1])), filter(lambda x: x[5] == 'Opened', orders['orders']))

            for order_id in open_order_ids:
                data = {
                    'owner': owner,
                    'id': order_id,
                    'asset': trader["asset"],
                }

                res = requests.post(f'http://{trader["endpoint"]}/cancel', json=data)

                if res.status_code != 200:
                    print(f'Trader #{trader["num"]} CANCEL ERROR\n{orders["errors"]}\n', flush=True)

            open_trades = False

        time.sleep(trader["loop_delay"])


if __name__ == '__main__':
    url = "34.68.113.9"
    name = 'AAPL'
    bid_mean = 98
    ask_mean = 102
    var = 5
    loop_delay = 0.1

    num_traders = 2

    traders = [{
        'num': i,
        'endpoint': url,
        'asset': name,
        'Bid mean': bid_mean,
        'Ask mean': ask_mean,
        'var': var,
        'loop_delay': loop_delay
    } for i in range(num_traders)]

    with Pool(num_traders) as pool:
        pool.map(run_trader, traders, chunksize=1)
