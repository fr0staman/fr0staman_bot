from aiogram.dispatcher.webhook import AnswerCallbackQuery
from aiogram.types import CallbackQuery

from core import config
from core.misc import dp, bot, _

from modules.inline_mode.callbacks import inline_callback
from utils.db_api import User


@dp.callback_query_handler(inline_callback.filter(action=['subscribe']))
async def callback_subscribe(query: CallbackQuery, callback_data: dict):
    if (await bot.get_chat_member(config.CHANNEL_ID, query.from_user.id)).status != 'left':
        if await User.user_exists(query.from_user):
            await User.subscribe_user(query.from_user)
            return AnswerCallbackQuery(query.id, _('Thank you!\nYou receive +100 kg forever 🎂'))
        else:
            return AnswerCallbackQuery(query.id, _('😭 There is no pig in your barn!'))
    else:
        return AnswerCallbackQuery(query.id, _('🐽 First of all - subscribe to the channel!'))


@dp.callback_query_handler(inline_callback.filter(action=['subscribe_gift']))
async def callback_subscribe(query: CallbackQuery, callback_data: dict):
    if (await bot.get_chat_member(config.CHANNEL_ID, query.from_user.id)).status != 'left':
        if await User.user_exists(query.from_user):
            await User.subscribe_user(query.from_user)
            await User.add_500_kg(query.from_user)
            return AnswerCallbackQuery(query.id, _('Thank you!\nYou receive +500kg today 🎂'))
        else:
            return AnswerCallbackQuery(query.id, _('😭 There is no pig in your barn!'))
    else:
        return AnswerCallbackQuery(query.id, _('🐽 First of all - subscribe to the channel!'))
