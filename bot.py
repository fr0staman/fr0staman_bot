from aiogram import Dispatcher

from core.load import load_modules
from core import config, misc


async def startup_webhook(dp: Dispatcher):
    await dp.bot.set_webhook(config.WEBHOOK_URL, max_connections=15)


async def startup_polling(dp: Dispatcher):
    await dp.bot.delete_webhook()


async def shutdown(dp: Dispatcher):
    misc.log.info("Bot disabled!")


def main():
    load_modules()

    misc.runner.on_shutdown(startup_polling)
    misc.runner.on_startup(startup_webhook, polling=False, webhook=True)
    misc.runner.on_startup(startup_polling, polling=True, webhook=False)

    misc.runner.on_shutdown(shutdown)

    if config.USE_WEBHOOK:
        misc.runner.start_webhook(**config.webhook_server)
    else:
        misc.runner.start_polling(timeout=0)


if __name__ == '__main__':
    main()
