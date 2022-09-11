from aiogram import types
from aiogram.dispatcher.webhook import SendMessage
from aiogram.utils.markdown import hlink

from core.misc import dp, bot, _


@dp.message_handler(commands=['test'])
async def cmd_testlang(message):
    return SendMessage(chat_id=message.chat.id,
                       text=_('Your name: {first_name}').format(first_name=message.from_user.first_name),
                       reply_to_message_id=message.message_id)


@dp.message_handler(commands=['lang'])
async def cmd_lang(message):
    return SendMessage(chat_id=message.chat.id,
                       text=message.from_user.language_code,
                       reply_to_message_id=message.message_id)


@dp.message_handler(commands=['id'], commands_prefix='!/#')
async def cmd_id(message):
    command, temp, msg_args = message.text.partition(' ')
    if msg_args:
        return SendMessage(message.chat.id,
                           hlink(_('your beauty'), f'tg://user?id={str(msg_args)}'))
    else:
        return SendMessage(message.chat.id, _('Give me a user_id as a argument :)'))


# @dp.message_handler(lambda m: m.chat.id == m.from_user.id, content_types=types.ContentType.VOICE)
# async def get_voice_id(message):
#    return SendMessage(message.chat.id, message.voice.file_id)


@dp.message_handler(lambda m: m.chat.id == m.from_user.id, content_types=types.ContentType.STICKER)
async def get_sticker_id(message):
    return SendMessage(message.chat.id, message.sticker.file_id)


@dp.message_handler(lambda m: m.chat.id == m.from_user.id, content_types=types.ContentType.ANIMATION)
async def get_gif_id(message):
    return SendMessage(message.chat.id, message.animation.file_id)


@dp.message_handler(commands=['i18n'], commands_prefix='!/#')
async def i18n_test(message):
    await message.reply(_('Pooping sobaka, crawling gusaka <b>{user}</b>').format(user=message.from_user.full_name))