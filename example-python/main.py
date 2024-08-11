#!python

import asyncio
import rmemstore_client

async def main():
    client = await rmemstore_client.client()
    futures = []
    for i in range(0, 100):
        futures.append(asyncio.create_task(client.put(bytes(f'123{i}', "utf-8"), f"hello{i}")))
    (results, _) = await asyncio.wait(futures)
    for result in results:
        result = result.result()
        if not result.ok:
            print(result)

    client.close()

if __name__ == "__main__":
    loop = asyncio.new_event_loop()
    loop.run_until_complete(main())
