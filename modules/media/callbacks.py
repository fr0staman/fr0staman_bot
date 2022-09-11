from aiogram.dispatcher.webhook import EditMessageCaption
from aiogram.types import CallbackQuery, InlineKeyboardMarkup
from aiogram.utils.callback_data import CallbackData

from core import config
from core.misc import dp, bot, _
from utils.db_api import User
voice_data = CallbackData('vote', 'action', 'id')


@dp.callback_query_handler(voice_data.filter(action=['allow_voice']))
async def callback_allow_voice(query: CallbackQuery, callback_data: dict):
    if query.from_user.id != config.CREATOR_ID:
        await query.answer('тікай з села - не твоя кнопка')
    else:
        await query.answer('принято, є таке!')
        url = 'fdjdjsfkkfjsdfj/' + str(query.message.message_id)
        user_id = int(callback_data['id'])
        await User.add_voice_group(url, user_id, 1, '')
        await User.add_user_kg(250, user_id)
        await bot.send_message(user_id, '<b>Поздравляем</b>, ваш хрюк принят!\n<b>Вы</b> получаете <b>250</b> кг для вашего ручного хряка :)')

        return EditMessageCaption(
            caption='принято от ' + str(user_id),
            message_id=query.message.message_id,
            chat_id=query.message.chat.id,
            reply_markup=InlineKeyboardMarkup()
        )


@dp.callback_query_handler(voice_data.filter(action=['disallow_voice']))
async def callback_disallow_voice(query: CallbackQuery, callback_data: dict):
    if query.from_user.id != config.CREATOR_ID:
        await query.answer(_('tikay s sela - it\'s not your button'))
    else:
        await query.answer('не принято, смотри логи')
        user_id = int(callback_data['id'])

        await bot.send_message(user_id, 'Ваш хрюк не принят :(\nПопробуйте ещё раз!')
        return EditMessageCaption(
            caption='не принято от ' + str(user_id),
            message_id=query.message.message_id,
            chat_id=query.message.chat.id,
            reply_markup=InlineKeyboardMarkup()
        )
