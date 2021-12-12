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
end_time = 1_000_000_000
fig = mpl.figure(style='yahoo')
ax = fig.add_subplot(111)

def animate(ival):
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
        return

    df = pd.DataFrame(columns=headers, data=res.json()['price']).set_index(['market_time'], drop=True)
    df.index = pd.to_datetime(df.index)

    ax.clear()
    mpl.plot(df, ax=ax, type='candle')


ani = animation.FuncAnimation(fig, animate, interval=1000)

mpl.show()