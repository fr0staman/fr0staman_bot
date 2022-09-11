import asyncio
import logging

from aiogram import Bot, Dispatcher
from aiogram.bot.api import TelegramAPIServer
from aiogram.utils.executor import Executor
from aiogram.types import ParseMode
from aiogram.contrib.middlewares.logging import LoggingMiddleware

from core.packages import PackagesLoader
from core import config
from middlewares.i18n import setup_middleware


# loop = asyncio.get_event_loop()

def get_botapi_server() -> TelegramAPIServer:
    if config.USE_WEBHOOK:
        if config.BOTAPI_HOST:
            return TelegramAPIServer.from_base(config.BOTAPI_HOST)


local_server = get_botapi_server()
bot_settings = {
    'token': config.BOT_TOKEN,
    'parse_mode': ParseMode.HTML,
}

if local_server:
    bot_settings['server'] = local_server

bot = Bot(**bot_settings
          # TODO:
          # Taking loop may to database crashing, recheck the "right" way
          # loop=loop
          )

dp = Dispatcher(bot=bot)
runner = Executor(dp, skip_updates=config.SKIP_UPDATES)
loader = PackagesLoader()

logging.basicConfig(level=logging.INFO)
log = logging.getLogger("fr0staman")

dp.middleware.setup(LoggingMiddleware())
dp.errors_handlers.once = True
i18n = setup_middleware(dp)

_ = i18n.gettext
__ = i18n.gettext
