import pandas as pd
import requests
import matplotlib.animation as animation
import mplfinance as mpl


asset = 'AAPL'
headers = ['market_time', 'low', 'high', 'open', 'close', 'volume']
candles = pd.DataFrame(columns=headers)
candles = candles.set_index(['market_time'], drop=True)
candles.index = pd.to_datetime(candles.index)
start_time = 0
end_time = 1_000_000_000_000

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
else:
    df = pd.DataFrame(columns=headers, data=res.json()['price']).set_index(['market_time'], drop=True)
    df.index = pd.to_datetime(df.index)
    mpl.plot(df, type='candle', style='yahoo', volume=True)
