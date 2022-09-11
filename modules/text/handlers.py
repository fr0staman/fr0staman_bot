from asyncio import sleep, ensure_future

from aiogram import types
from aiogram.dispatcher.webhook import SendMessage, SendSticker, SendVoice, DeleteMessage
from aiogram.utils.markdown import hitalic

from core.config import CREATOR_ID
from core.misc import bot, dp, _
from utils.db_api import Tech
from modules.admin.keyboards import check_subscribe, to_channel


@dp.my_chat_member_handler(lambda c: c.chat.id == c.from_user.id and c.new_chat_member.status == 'kicked')
async def who_blocks(chat_member: types.ChatMemberUpdated):
    await Tech.delete_user(chat_member.from_user.id)


@dp.message_handler(lambda m: m.chat.id == CREATOR_ID, commands=['toallgroups'], commands_prefix='!/#')
async def cmd_get_groups(message):
    groups = await Tech.get_all_groups()


@dp.message_handler(lambda m: m.chat.id == CREATOR_ID, commands=['toallusers'], commands_prefix='!/#')
async def cmd_get_groups(message):
    users = await Tech.get_all_users()
    for user_id in users:
        try:
            await bot.send_message(user_id,
                                   _('<b>Important!</b>\n\n'
                                     'Special thanks to you, for a destiny in life of this bot!\n'
                                     'Just take your gift :)\n'),
                                   disable_web_page_preview=True,
                                   reply_markup=to_channel())
        except Exception as e:
            if str(e) == 'Forbidden: bot was blocked by the user':
                await Tech.delete_user(user_id)
            else:
                await bot.send_message(CREATOR_ID, str(e))
        await sleep(0.1)


@dp.message_handler(commands=['hryak'], commands_prefix='!/#')
async def cmd_hryak(message):
    return SendMessage(message.chat.id, _('интересно, а у хряков есть свой Слава Марлов, токо вместо басов хрюканья?'))


@dp.message_handler(commands=['print', 'p'], commands_prefix='!/#')
async def cmd_print(message):
    command, temp, msg_args = message.text.partition(' ')
    try:
        ensure_future(message.delete())
    except:
        pass
    if msg_args:
        if message.reply_to_message:
            return SendMessage(message.chat.id, hitalic(msg_args),
                               reply_to_message_id=message.reply_to_message.message_id)
        else:
            return SendMessage(message.chat.id, hitalic(msg_args))
    else:
        return SendMessage(message.chat.id, _('Write your text as argument!'))


@dp.message_handler(lambda m: m.reply_to_message, commands=['pidor'], commands_prefix='!/#')
async def cmd_pidor_reply(message):
    await bot.send_message(message.chat.id, _('you are pidor'),
                           reply_to_message_id=message.reply_to_message.message_id)
    return DeleteMessage(message.chat.id, message.message_id)


@dp.message_handler(commands=['pidor'], commands_prefix='!/#')
async def cmd_pidor(message):
    return DeleteMessage(message.chat.id, message.message_id)


@dp.message_handler(content_types=['text'])
async def check_text(message):
    # await insert_groups(message.chat.id)
    if message.text == '@fr0staman_bot':
        return SendMessage(message.chat.id, 'https://t.me/fr0staman_chat/24007', reply_to_message_id=message.message_id)
    elif message.text.lower() == 'вита':
        return SendMessage(message.chat.id, 'уёбище', reply_to_message_id=message.message_id)
    elif message.text.lower() == 'хорни' or message.text.lower == 'horny':
        return SendMessage(message.chat.id, hitalic('go to horny jail.'), reply_to_message_id=message.message_id)
    elif message.text.lower() == 'димас':
        return SendMessage(message.chat.id, _('pidaras'), reply_to_message_id=message.message_id)
    elif message.text.lower() == 'бдсм' and message.reply_to_message:
        return SendSticker(message.chat.id,
                           'CAACAgIAAx0CWjbDqQACJqVhBXDPsHT3uscpSlWcQTQxhjgetgACdAEAAntOKhC7YDsAAWimi98gBA',
                           reply_to_message_id=message.reply_to_message.message_id)
    elif message.text.lower() == 'хрюкни' and message.reply_to_message:
        return SendVoice(message.chat.id,
                         'AwACAgIAAxkBAAIfv2Eep89pMun_Qq3u-o_UdS997Bx9AAIsEgACErX4SBRdIvQwnUdhIAQ',
                         reply_to_message_id=message.reply_to_message.message_id)
