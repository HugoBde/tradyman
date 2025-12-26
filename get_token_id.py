#!/usr/bin/python
import datetime
import requests

now = datetime.datetime.now()
slug_suffix = int(
    datetime.datetime(
        now.year, now.month, now.day, now.hour, now.minute // 15 * 15, 0
    ).timestamp()
)

print(f"btc-updown-15m-{slug_suffix}")
token_ids = eval(
    requests.get(
        f"https://gamma-api.polymarket.com/events/slug/btc-updown-15m-{slug_suffix}"
    ).json()["markets"][0]["clobTokenIds"]
)
print(token_ids[0])
print(token_ids[1])
