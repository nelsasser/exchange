import time
import pandas as pd
import requests

if __name__ == '__main__':
    asset = 'aapl'

    headers = ['market_time', 'low', 'high', 'open', 'close', 'volume']

    candles = pd.DataFrame(columns=headers)

    start_time = 0
    end_time = 1_000_000_000

    while True:
        res = requests.post("http://34.68.113.9/price", json={
            'asset': asset,
            'start_time': start_time,
            'end_time': end_time,
        })

        if res.status_code != 200:
            try:
                print('ERROR:', res.json()['errors'])
            except:
                print('ERROR!!!:', res.text)

