import asyncio

gather = asyncio.gather
create_task = asyncio.create_task
sleep_ms = asyncio.sleep_ms

def run_until_complete(tasks):
    asyncio.run(tasks)

