from random import choice

from aiogram import types
from aiogram.dispatcher.webhook import SendMessage, SendVoice
from aiogram.utils.markdown import hitalic, hlink

from core.config import CREATOR_ID, CONTENT_CHANNEL_ID
from core.misc import bot, dp, _

# @dp.message_handler(commands=['pig'], commands_prefix='!/#')
# async def cmd_random_pig(message):
#    await bot.send_animation(message.chat.id, choice(hryak_gifs))


# @dp.message_handler(lambda m: m.sticker.set_name in HRYAK_PACKS, content_types=types.ContentType.STICKER)
# async def sticker_hryak(message):
#    return SendMessage(message.chat.id, hitalic(choice(hryak_msgs)), reply_to_message_id=message.message_id)
from modules.media.keyboards import voice_reply_markup


@dp.message_handler(lambda m: m.chat.id == m.from_user.id, content_types=types.ContentType.VOICE)
async def voice_hryak(message):
    user_id = message.from_user.id
    # mention = hlink(message.from_user.first_name + ' ' + message.from_user.last_name, f'tg://user?id={str(user_id)}')
    await bot.send_message(user_id, _('Thank you!\nI will be sure to let you, when your brilliant grunt will be accepted!'))
    return SendVoice(CONTENT_CHANNEL_ID,
                     str(message.voice.file_id), caption=str(user_id), reply_markup=voice_reply_markup(user_id))


@dp.message_handler(content_types=['sticker'])
async def react_stickers(message):
    return
    # await bot.send_message(message.chat.id, message.sticker.file_id)
    # await bot.send_message(message.chat.id, message.sticker.file_unique_id)
    # await bot.send_message(message.chat.id, "Кинь свиню.")
