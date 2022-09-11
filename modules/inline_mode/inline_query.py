import re

from aiogram import types
from aiogram.dispatcher.webhook import AnswerInlineQuery, SendMessage
from aiogram.utils.markdown import hbold, hitalic

from core.config import CREATOR_ID

from core.misc import bot, _
from modules.base.keyboards import game_inline
from modules.base.lang import lng
from modules.hryak.consts import top_10_header

from modules.inline_mode.hryak_text import calculate_hryak_size, get_emoji
from modules.inline_mode.keyboards import top_10_create, top_10_chat, top_10_win_private, hryak_battle_markup, \
    give_name_markup, inline_hryak_of_the_day_markup
from utils.db_api import Inline, HandPig


async def hryak_inline(query):
    try:
        user_id = query.from_user.id
        user_lang = query.from_user.language_code
        user_name = query.from_user.first_name

        hryak_size, status, winrate, the_name = await HandPig.get_hryak_size(user_id)

        if hryak_size == -1:
            hryak_size = calculate_hryak_size(user_id)
            await HandPig.insert_hryak(user_lang, user_id, user_name, hryak_size)

            hryak_size, status, winrate, the_name = await HandPig.get_hryak_size(user_id)
        if winrate == 0:
            winrate = hitalic(_('Not enough battles...'))
        else:
            winrate = hbold(str(winrate) + '%')
        top_10_text = await HandPig.get_top10_global(user_lang)
        inline_hryak_text = \
            _('Weight of your piggy {weight} kg {emoji}') \
                .format(weight=hbold(hryak_size), emoji=get_emoji(hryak_size))
        if status == 2:
            inline_hryak_text = \
                inline_hryak_text + hitalic(_('\n\nThis user supported bot developing.\nThanks!'))

        hryak_size_markup = top_10_create(user_id, user_lang)
        top_10_markup = top_10_chat(user_id, user_lang)
        if hasattr(query, 'chat_type'):
            if query.chat_type == 'private':
                hryak_size_markup = types.ReplyKeyboardMarkup()
                top_10_markup = top_10_win_private(lng(user_lang))
            elif query.chat_type == 'channel':
                hryak_size_markup = game_inline(user_lang)
        else:
            hryak_size_markup = types.ReplyKeyboardMarkup()
        inline_result = [
            types.InlineQueryResultArticle(
                id='1',
                title=_('⚔ Duel of boars ⚔'),
                description=_('@fr0staman_chat for fattest'),
                thumb_url="https://static.46.56.55.162.clients.your-server.de/host/upload/1_fight_200x200.jpg",
                thumb_width=200,
                thumb_height=200,
                input_message_content=types.InputTextMessageContent(
                    message_text=
                    _('🛡 Boar {name} challenge to a duel! 🛡\n'  # make randomize
                      'Win percentage: {winrate}\n'
                      'Preliminary weighing: {weight} kg')
                        .format(name=hbold(the_name),
                                winrate=winrate,
                                weight=hbold(hryak_size)),
                    disable_web_page_preview=True
                ),
                reply_markup=hryak_battle_markup(user_id, user_lang),
            ),
            types.InlineQueryResultArticle(
                id='2',
                title=top_10_header[lng(user_lang)],  # DONT FORGET
                description=_('Best of the best!'),
                thumb_url="https://static.46.56.55.162.clients.your-server.de/host/upload/2_top_200x200.jpg",
                thumb_width=200,
                thumb_height=200,
                input_message_content=types.InputTextMessageContent(
                    message_text=top_10_text,
                    disable_web_page_preview=True
                ),
                reply_markup=top_10_markup
            ),
            types.InlineQueryResultArticle(
                id='3',
                title=_('🐖 find out your pig mass'),
                description=_('Weight: {weight} kg').format(weight=hryak_size),
                thumb_url="https://static.46.56.55.162.clients.your-server.de/host/upload/3_take_weight_200x200.jpg",
                thumb_width=200,
                thumb_height=200,
                input_message_content=types.InputTextMessageContent(
                    message_text=inline_hryak_text,
                    disable_web_page_preview=True
                ),
                reply_markup=hryak_size_markup
            ),
        ]
        await query.answer(inline_result, cache_time=0)
    except Exception as e:
        inline_result = [
            types.InlineQueryResultArticle(
                id='1',
                title=_('Error!'),
                description=_('This bug has already been reported.'),
                thumb_url="https://static.46.56.55.162.clients.your-server.de/host/upload/9_error_200x200.jpg",
                thumb_width=200,
                thumb_height=200,
                input_message_content=types.InputTextMessageContent(
                    message_text=_('An error has occurred :(\n'
                                   '\n'
                                   'Bug report has already been sent!')
                )
            )
        ]
        await query.answer(inline_result, cache_time=0)
        return SendMessage(CREATOR_ID, str(query) + '\n' + str(e))


async def oc_inline(query):
    user_id = query.from_user.id
    user_lang = query.from_user.language_code
    size = calculate_hryak_size(user_id)

    clock_cpu = (((size + user_id) % 42) + 19) / 10
    clock_ram = ((size + user_id) % 4533) + 1333
    hashrate = ((size + user_id) % 12800) / 100
    clock_ram = clock_ram + int((266.67 - (clock_ram % 266.67)))
    cpu_emoji = '🧊'
    if clock_cpu > 5.5:
        cpu_emoji = '🌋'
    elif clock_cpu > 4.9:
        cpu_emoji = '💥'
    elif clock_cpu > 4.7:
        cpu_emoji = '💣'
    elif clock_cpu > 4.4:
        cpu_emoji = '🧨'
    elif clock_cpu > 3.9:
        cpu_emoji = '♨'
    ram_emoji = '🧊'
    if clock_ram > 5300:
        ram_emoji = '🌋'
    elif clock_ram > 5000:
        ram_emoji = '💥'
    elif clock_ram > 4600:
        ram_emoji = '💣'
    elif clock_ram > 4000:
        ram_emoji = '🧨'
    elif clock_ram > 3600:
        ram_emoji = '♨'
    hashrate_emoji = '🐢'
    if hashrate > 119:
        hashrate_emoji = '🔥'
    elif hashrate > 109:
        hashrate_emoji = '🚝'
    elif hashrate > 99:
        hashrate_emoji = '🚜'
    elif hashrate > 79:
        hashrate_emoji = '🚛'
    elif hashrate > 59:
        hashrate_emoji = '⛹'
    elif hashrate > 39:
        hashrate_emoji = '🧗'
    elif hashrate > 19:
        hashrate_emoji = '🤸'
    inline_overclocking = [
        types.InlineQueryResultArticle(
            id='1',
            title=_('💥 find out overclocking of your processor 💥'),
            thumb_url="https://static.46.56.55.162.clients.your-server.de/host/upload/8_1_cpu_200x200.jpg",
            thumb_width=200,
            thumb_height=200,
            input_message_content=types.InputTextMessageContent(
                message_text=_('Overclocking of your processor {cpu_clock} GHz {cpu_emoji}')
                    .format(cpu_clock=hbold(clock_cpu), cpu_emoji=cpu_emoji)
            )
        ),
        types.InlineQueryResultArticle(
            id='2',
            title=_('🐏 find out overclocking of your RAM 🐏'),
            thumb_url="https://static.46.56.55.162.clients.your-server.de/host/upload/8_2_ram_200x200.jpg",
            thumb_width=200,
            thumb_height=200,
            input_message_content=types.InputTextMessageContent(
                message_text=_('Overclocking of your RAM {ram_clock} Mhz {ram_emoji}')
                    .format(ram_clock=hbold(clock_ram), ram_emoji=ram_emoji)
            )
        ),
        types.InlineQueryResultArticle(
            id='3',
            title=_('🎢 find out hashrate of your GPU 🎢'),
            thumb_url="https://static.46.56.55.162.clients.your-server.de/host/upload/8_3_gpu_200x200.jpg",
            thumb_width=200,
            thumb_height=200,
            input_message_content=types.InputTextMessageContent(
                message_text=_('Hashrate of your videocard {gpu_hashrate} MH/s {gpu_emoji}')
                    .format(gpu_hashrate=hbold(hashrate), gpu_emoji=hashrate_emoji)
            )
        ),
    ]
    return AnswerInlineQuery(query.id, inline_overclocking, cache_time=0)


async def hryak_name_inline(query):
    user_id = query.from_user.id
    user_lang = query.from_user.language_code

    size, status, f_name, name = await HandPig.get_about_user(user_id)
    inline_name_query = [
        types.InlineQueryResultArticle(
            id='1',
            title=_('Keep going!'),
            description=_('Press space and type future name of your boar!'),
            thumb_url="https://static.46.56.55.162.clients.your-server.de/host/upload/4_name_typing_200x200.jpg",
            thumb_width=200,
            thumb_height=200,
            input_message_content=types.InputTextMessageContent(
                message_text=_('Name of your boar: {name}\n'
                               '\n'
                               'If you want change name of your pig, just type:\n'
                               '\n'
                               '@fr0staman_bot name *pig_name*')
                    .format(name=hbold(name))
            )
        )
    ]
    return AnswerInlineQuery(query.id, inline_name_query, cache_time=0)


async def hryak_name_inline_give(query, future_name):
    user_id = query.from_user.id
    user_lang = query.from_user.language_code

    future_name = re.sub(r'[<>\'\"@]', '', future_name)
    size, status, f_name, t_name = await HandPig.get_about_user(user_id)
    inline_name_give_query = [
        types.InlineQueryResultArticle(
            id='1',
            title=_('{future_name}').format(future_name=future_name),
            description=_('click to set a name'),
            thumb_url="https://static.46.56.55.162.clients.your-server.de/host/upload/5_name_success_200x200.jpg",
            thumb_width=200,
            thumb_height=200,
            input_message_content=types.InputTextMessageContent(
                message_text=
                _('Todays name of your pig: {past_name}\n'
                  'Future name of your boar: {future_name}\n'
                  '\n'
                  'Just click the button!')
                    .format(past_name=hbold(t_name),
                            future_name=hbold(future_name))
            ),
            reply_markup=give_name_markup(user_id, user_lang, future_name)
        )
    ]
    return AnswerInlineQuery(query.id, inline_name_give_query, cache_time=0)


async def hryak_of_the_day(query):
    user_lang = query.from_user.language_code

    inline_name_give_query = [
        types.InlineQueryResultArticle(
            id='1',
            title=_('Pig of the day!'),
            description=_('who is the hryak of your chat?'),
            thumb_url="https://static.46.56.55.162.clients.your-server.de/host/upload/6_pigoftheday_200x200.jpg",
            thumb_width=200,
            thumb_height=200,
            input_message_content=types.InputTextMessageContent(
                message_text=_('Click the button to catch the hryak of your chat...'),
            ),
            reply_markup=inline_hryak_of_the_day_markup(user_lang)
        ),

    ]
    return AnswerInlineQuery(query.id, inline_name_give_query, cache_time=0)


async def hryak_voice_hru(query):
    result = await Inline.get_all_voices()
    inline_array = []
    for i in result:
        url = f"https://t.me/{i['url']}"
        caption = _('hruk of the death №{number}').format(number=str(i['id']))
        inline_array.append(
            types.InlineQueryResultVoice(
                id=i['id'],
                title=caption,
                voice_url=url
            )
        )
    return AnswerInlineQuery(query.id, inline_array, cache_time=30)
