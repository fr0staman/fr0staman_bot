import re
import typing

from random import randrange

from aiogram import types
from aiogram.dispatcher.webhook import SendMessage, EditMessageText
from aiogram.types import InlineKeyboardButton, InlineKeyboardMarkup, CallbackQuery
from aiogram.utils.callback_data import CallbackData
from aiogram.utils.exceptions import MessageNotModified
from aiogram.utils.markdown import hbold, hitalic, hlink

from core.misc import dp, bot, _, __

from modules.base.date import get_date, get_timediff
from modules.base.keyboards import game_inline
from utils.db_api import ChatPig

next_to = CallbackData('vote', 'action')
query_num = {}


async def top_10_get_text(chat_id, lim=0):
    top10_setting = await ChatPig.get_top10_setting((chat_id,))
    if bool(top10_setting):
        setting = top10_setting[0]['top10_setting']
    else:
        setting = 0

    if setting == 0:
        query_value = (chat_id, chat_id, lim)
        exit_value = await ChatPig.get_top10_chat_nr(query_value)
    else:
        query_value = (chat_id, setting, chat_id, setting, lim)
        exit_value = await ChatPig.get_top10_chat_st(query_value)

    message_text = ''
    i = lim
    allcount = 0
    for value in exit_value:
        allcount = value['mycount']
        name = value['name'] if value['name'] != '' else 'хряк'
        i = i + 1
        message_params = (hbold(i), name, hbold(value['kg_now']))
        message_text = message_text + ('%s. %s - %s кг\n' % message_params)
    return message_text, len(exit_value), allcount


def top_10_list(rowcount, allcount):
    if rowcount == allcount:
        return ''
    elif rowcount == 50 and allcount % 50 != 0:  # if query before not exists, in future
        return InlineKeyboardMarkup().row(
            InlineKeyboardButton('⬅️', callback_data=next_to.new(action='left')),
            InlineKeyboardButton('➡️', callback_data=next_to.new(action='right'))
        )
    elif rowcount < 50:
        return InlineKeyboardMarkup().row(
            InlineKeyboardButton('⬅️', callback_data=next_to.new(action='left'))
        )
    else:
        return InlineKeyboardMarkup().row(
            InlineKeyboardButton('⬅️', callback_data=next_to.new(action='left')),
            InlineKeyboardButton('➡️', callback_data=next_to.new(action='right'))
        )


@dp.message_handler(lambda m: m.chat.id != m.from_user.id, commands=['grow'], commands_prefix='!/#')
async def cmd_grow(message):

    user_name = message.from_user.first_name
    user_id = message.from_user.id
    chat_id = message.chat.id

    check_day = await ChatPig.check_user_day((chat_id, user_id, get_date()))

    if not bool(check_day):
        current_valu = 0
        grow = randrange(-2, +20)
        diya = _('gained')
        if grow < 1:
            grow = randrange(-20, -1)
            diya = _('lost')
        mention = hlink(user_name, f'tg://user?id={str(user_id)}')

        hryak_values = await ChatPig.get_hryak_values((chat_id, user_id))

        if not bool(hryak_values):
            if grow < 1:
                grow = 1
            val = (chat_id, user_id, grow, get_date(), user_name)
            await ChatPig.add_new_hryak(val)
            await bot.send_message(message.chat.id,
                                   _('Welcome to the game, {mention}!\n'
                                     'Keep using /grow command, to grow your own pig :)')
                                   .format(mention=mention),
                                   )
        else:
            current_valu = hryak_values[0]['kg_now']
            user_name = hryak_values[0]['name']

            if current_valu + grow < 1:
                current_valu = 0
                grow = 1
                diya = _('lost')
            update_value = (current_valu + grow, get_date(), user_name, chat_id, user_id)
            await ChatPig.update_hryak(update_value)

        if user_name == '':
            user_name = _('unnamed pig')

        return SendMessage(message.chat.id,
                           _('{mention}, your {name} {action} on {value} kg fat!\n\n'
                             'Weight of your pig: {current} kg.').format(
                               mention=mention,
                               name=hbold(user_name),
                               action=hbold(diya),
                               value=hbold(abs(grow)),
                               current=hbold(current_valu + grow))
                           )
    else:
        timediff = get_timediff()

        to_minutes = __('{minutes} minute', '{minutes} minutes', timediff['minutes']).format(
            minutes=timediff['minutes'])
        if timediff['hours'] == 0:
            next_feed = _('Next feeding in {to_minutes}.').format(to_minutes=to_minutes)
        else:
            to_hours = __('{hours} hour', '{hours} hours', timediff['hours']).format(hours=timediff['hours'])
            next_feed = _('Next feeding in {to_hours} and {to_minutes}.').format(to_hours=to_hours,
                                                                                 to_minutes=to_minutes)

        return SendMessage(message.chat.id,
                           _('You already fed pig today!\n{next_feed}').format(next_feed=hitalic(next_feed)))


@dp.message_handler(lambda m: m.chat.id != m.from_user.id, commands=['name'], commands_prefix='!/#')
async def cmd_add_name(msg: types.Message):
    command, _t, msg_args = msg.text.partition(' ')

    chat_id = msg.chat.id
    user_id = msg.from_user.id
    user_name = msg.from_user.first_name

    if msg_args:
        if len(msg_args) > 64:
            return SendMessage(chat_id, _('The tag does not fit more than 64 letters!'))
        msg_args = re.sub(r'[<>\'\"@]', '', msg_args)
        msg_args = msg_args.replace("\n", ' ')

        if msg_args == '':
            msg_args = 'позорный хряк'

        hryak_name = (msg_args, user_name, str(chat_id), user_id)

        await ChatPig.set_hryak_name(hryak_name)
        return SendMessage(chat_id,
                           _('New name of your 🐷: \n {new_name}').format(new_name=hitalic(msg_args)),
                           disable_web_page_preview=True, )
    else:
        hryak_info = (chat_id, user_id)
        name_value = await ChatPig.get_hryak_name(hryak_info)

        if not bool(name_value):
            return SendMessage(chat_id, _('😭 There is no pig in your barn!'))
        else:
            hryak_name = name_value[0]['name']
            if hryak_name == '':
                return SendMessage(chat_id,
                                   _('Your chat pig doesnt have a name!\nTo name your pig, just call /name pig_name, anything :)')
                                   )

            return SendMessage(chat_id,
                               _('Name of your pig: \n {name}\n\nChange name - /name pig_name.')
                               .format(name=hitalic(hryak_name)),
                               disable_web_page_preview=True)


@dp.message_handler(lambda m: m.chat.id != m.from_user.id, commands=['my'], commands_prefix='!/#')
async def cmd_my_hryak(msg):
    chat_id = msg.chat.id
    user_id = msg.from_user.id

    qu_args = (chat_id, user_id)
    hryak_about = await ChatPig.get_hryak_chrs(qu_args)

    if not bool(hryak_about):
        return SendMessage(chat_id, _('😭 There is no pig in your barn!'))

    kg = hryak_about[0]['kg_now']
    hryak_name = hryak_about[0]['name']
    if hryak_name == '':
        hryak_name = _('unnamed pig')

    return SendMessage(chat_id,
                       _('Your 🐷 {name}\nHas weight {current}')
                       .format(name=hbold(hryak_name), current=hbold(kg)),
                       disable_web_page_preview=True)


@dp.message_handler(lambda m: m.chat.id != m.from_user.id, commands=['top'], commands_prefix='!/#')
async def top_10(message):
    (message_text, rowcount, allcount) = await top_10_get_text(message.chat.id)
    if message_text == '':
        return SendMessage(message.chat.id, _('😭 There is no pig in your barn!'))
    header = _('🐷 Top 50 schweinehryaks 🐷\n\n')
    header = header + str(message_text)
    return SendMessage(message.chat.id, header, disable_web_page_preview=True,
                       reply_markup=top_10_list(rowcount, allcount))


@dp.callback_query_handler(next_to.filter(action=['left', 'right']))
async def callback_top10(query: CallbackQuery, callback_data: typing.Dict[str, str]):
    await query.answer()
    chat_id = query.message.chat.id
    callback_data_action = callback_data['action']
    query_number = query_num.get(chat_id, 0)
    if callback_data_action == 'right':
        query_number += 50
    else:
        if query_number >= 50:
            query_number -= 50
    query_num[chat_id] = query_number
    message_text, rowcount, allcount = await top_10_get_text(query.message.chat.id, query_number)

    if message_text == '':
        return True
    header = hbold(_('🐷 Top 10 pig beasts 🐷\n\n'))
    header = header + str(message_text)
    return EditMessageText(
        header,
        chat_id,
        query.message.message_id,
        reply_markup=top_10_list(rowcount, allcount),
        disable_web_page_preview=True
    )


@dp.errors_handler(exception=MessageNotModified)
async def message_not_modified_handler(update, error):
    return True


@dp.message_handler(lambda m: m.chat.id == m.from_user.id, commands=['top', 'my', 'grow'],
                    commands_prefix='!/#')
async def cmd_top_user(message):
    return SendMessage(message.chat.id, _('Game "Grow The Pig" - only for chats!'))


@dp.message_handler(lambda m: m.chat.id == m.from_user.id, commands=['name'], commands_prefix='!/#')
async def cmd_add_name(message):
    command, temp, msg_args = message.text.partition(' ')
    if msg_args:
        if len(msg_args) > 64:
            return SendMessage(message.chat.id, _('The tag does not fit more than 64 letters!'))
        msg_args = re.sub(r'[<>\'\"@]', '', msg_args)
        msg_args = msg_args.replace("\n", ' ')
        if msg_args == '':
            msg_args = 'позорный хряк'

        hryak_name = (msg_args, message.from_user.first_name, message.from_user.id)
        await ChatPig.set_inline_name(hryak_name)

        return SendMessage(message.chat.id,
                           _('New name of your 🐷: \n {new_name}').format(new_name=hitalic(msg_args)))

    else:
        hryak_info = (message.from_user.id,)
        name_value = await ChatPig.get_inline_name(hryak_info)

        if len(name_value) == 0:
            return SendMessage(message.chat.id, _('😭 There is no pig in your barn!'))
        else:
            hryak_name = name_value[0]['name']
            if hryak_name == '':
                return SendMessage(message.chat.id,
                                   _('Your chat pig has no name!\n'
                                     'To name your pig, then /name his_name, just for fun :)'))

            return SendMessage(message.chat.id,
                               _('Name of your pig: \n {name}\n\nChange name - /name pig_name.').format(
                                   name=hitalic(hryak_name)))


@dp.message_handler(commands=['game'], commands_prefix='!/#')
async def cmd_game(message):
    return SendMessage(message.chat.id, _('Game "Grow The Pig"\n\n'
                                          'Commands:\n'
                                          '/grow - once a day feed your pig\n'
                                          '/name - name your piggy\n'
                                          '/my   - weigh and listen\n'
                                          '/top  - top of the fattest pigs in your barn '),
                       reply_markup=game_inline(message.from_user.language_code))
