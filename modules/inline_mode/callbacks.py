from asyncio import sleep
from random import randrange

from aiogram.dispatcher.webhook import EditMessageReplyMarkup, AnswerCallbackQuery, EditMessageText
from aiogram.types import CallbackQuery
from aiogram.utils.markdown import hbold, hitalic, hlink

from core.misc import dp, bot, _
from modules.base.date import get_date

from modules.inline_mode.consts import inline_callback, inline_callback_name
from modules.inline_mode.hryak_text import calculate_hryak_size
from modules.inline_mode.keyboards import top_10_chat, top_10_win, top_10_global, top_10_win_private, \
    top_10_global_private
from utils.db_api import HandPig


@dp.callback_query_handler(inline_callback.filter(action=['find_hryak']))
async def find_hryak(query: CallbackQuery, callback_data: dict):
    f_name, user_id = await HandPig.get_hryak_chat(query.chat_instance)
    if f_name == -1:
        status = await HandPig.add_hryak_chat(query.chat_instance)
        if status == -1:
            return AnswerCallbackQuery(query.id, _('😭 There is no chat pig in your barn!'))
        await query.answer()
        f_name, user_id = await HandPig.get_hryak_chat(query.chat_instance)
        mention = hlink(f_name, f'tg://user?id={str(user_id)}')
        await bot.edit_message_text(
            hitalic(_('Deploying schweinelocators...')),
            inline_message_id=query.inline_message_id,
        )
        await sleep(2)
        await bot.edit_message_text(
            hitalic(_('Hoof marks founded...')),
            inline_message_id=query.inline_message_id,
        )
        await sleep(2)
        await bot.edit_message_text(
            hbold(_('SCHWEINELOCATORS INSANELY SCREAMING!!')),
            inline_message_id=query.inline_message_id,
        )
        await sleep(2)
        return EditMessageText(
            _('🐷 <b>ATTENTION!</b> 🐷\n'
              '\n'
              '<b>Pig of the day</b> is announced {mention}!')
                .format(mention=mention),
            inline_message_id=query.inline_message_id,
        )
    elif f_name == -2:
        return AnswerCallbackQuery(query.id,
                                   _('😭 There is no pig in your barn!'))
    else:
        await query.answer()
        return EditMessageText(
            _('Position of the schweinelocators told us - <b>pig of the day</b> is {name}!')
                .format(name=hbold(f_name)),
            inline_message_id=query.inline_message_id,
        )


@dp.callback_query_handler(inline_callback_name.filter(action=['give_name']))
async def callback_top10_global_private(query: CallbackQuery, callback_data: dict):
    if int(callback_data['id']) == query.from_user.id:
        await HandPig.update_name(int(callback_data['id']), callback_data['name'])
        await query.answer(_('✨ Name changed successfully!'))
        return EditMessageText(
            _('🐷 Your pig is now called:\n {new_name} ')
                .format(new_name=hitalic(callback_data['name'])),
            inline_message_id=query.inline_message_id
        )
    else:
        return AnswerCallbackQuery(query.id, _('🤕 This button is not for you!'))


@dp.callback_query_handler(inline_callback.filter(action=['top10_global_private']))
async def callback_top10_global_private(query: CallbackQuery):
    await query.answer()
    top_10_g_text = await HandPig.get_top10_global(query.from_user.language_code)
    return EditMessageText(
        top_10_g_text,
        inline_message_id=query.inline_message_id,
        reply_markup=top_10_win_private(query.from_user.language_code))


@dp.callback_query_handler(inline_callback.filter(action=['top10_win_private']))
async def callback_top10_win_private(query: CallbackQuery, callback_data: dict):
    await query.answer()
    top10_win_text = await HandPig.get_top10_win(query.from_user.language_code)
    return EditMessageText(
        top10_win_text,
        inline_message_id=query.inline_message_id,
        reply_markup=top_10_global_private(query.from_user.language_code)
    )


@dp.callback_query_handler(inline_callback.filter(action=['add_chat']))
async def callback_top10_add(query: CallbackQuery, callback_data: dict):
    if int(callback_data['id']) == query.from_user.id:
        await HandPig.add_group_to_user(query.chat_instance, query.from_user.id)
        await query.answer(_('😎 You are in the rating of your chat!'))
        return EditMessageReplyMarkup(inline_message_id=query.inline_message_id, reply_markup='')
    else:
        return AnswerCallbackQuery(query.id, _('🤕 This button is not for you!'))


@dp.callback_query_handler(inline_callback.filter(action=['top10_global']))
async def callback_top10_global(query: CallbackQuery):
    await query.answer()
    top_10_g_text = await HandPig.get_top10_global(query.from_user.language_code)
    return EditMessageText(
        top_10_g_text,
        inline_message_id=query.inline_message_id,
        reply_markup=top_10_chat(query.from_user.id, query.from_user.language_code))


@dp.callback_query_handler(inline_callback.filter(action=['top10_chat']))
async def callback_top10_chat(query: CallbackQuery, callback_data: dict):
    await query.answer()
    await HandPig.add_group_to_user(query.chat_instance, int(callback_data['id']))
    top_10_chat_text = await HandPig.get_top10_chat(query.from_user.language_code, query.chat_instance)
    return EditMessageText(
        top_10_chat_text,
        inline_message_id=query.inline_message_id,
        reply_markup=top_10_win(query.from_user.language_code),
    )


@dp.callback_query_handler(inline_callback.filter(action=['top10_win']))
async def callback_top10_win(query: CallbackQuery, callback_data: dict):
    await query.answer()
    top10_win_text = await HandPig.get_top10_win(query.from_user.language_code)
    return EditMessageText(
        top10_win_text,
        inline_message_id=query.inline_message_id,
        reply_markup=top_10_global(query.from_user.language_code)
    )


BLOCKED_USERS = [802623513, 377953539, 1948871519, 580867121, 424131513]


@dp.callback_query_handler(inline_callback.filter(action=['start_duel']))
async def start_duel(query: CallbackQuery, callback_data: dict):
    called_user_id = query.from_user.id
    called_lang = query.from_user.language_code
    called_user_name = query.from_user.first_name
    if called_user_id == int(callback_data['id']):
        return AnswerCallbackQuery(query.id, _('🤕 This button is not for you!'))
    if called_user_id in BLOCKED_USERS:
        return AnswerCallbackQuery(query.id, _('🤕 This button is not for you!'))
    first_names, hryak_sizes, users_id = await HandPig.get_name_n_size(int(callback_data['id']), called_user_id)

    if users_id == 0:
        return AnswerCallbackQuery(query.id, _('😭 There is no pig in your barn!'))

    if hryak_sizes == -1:
        hryak_size = calculate_hryak_size(called_user_id)
        await HandPig.insert_hryak(called_lang,
                                   called_user_id,
                                   called_user_name,
                                   hryak_size)
        first_names, hryak_sizes, users_id = await HandPig.get_name_n_size(int(callback_data['id']), called_user_id)

    await query.answer()
    await bot.edit_message_reply_markup(inline_message_id=query.inline_message_id, reply_markup='')
    status = 0
    random_1 = hryak_sizes[0]
    random_2 = hryak_sizes[1]
    if random_1 // random_2 > 5:
        random_2 = random_1 // 5
    elif random_2 // random_1 > 5:
        random_1 = random_2 // 5
    first = randrange(0, random_1)
    second = randrange(0, random_2)

    if first > second:
        status = 1
        if first >= (hryak_sizes[0] * 95) // 100:
            status = 3
            if first >= (hryak_sizes[0] * 99) // 100:
                status = 5
    elif first < second:
        status = 2
        if second >= (hryak_sizes[1] * 95) // 100:
            status = 4
            if second >= (hryak_sizes[1] * 99) // 100:
                status = 6
    else:
        status = 0
    await sleep(randrange(2, 6))
    damage_first = hryak_sizes[0] // 8
    damage_second = hryak_sizes[1] // 8
    if status == 1:
        message_text = _('🎊 Winner is {winner_name}! 🎊\n'
                         '\n'
                         'He receives {diff} kg, but {looser_name} - loses fat, and its all...\n'
                         '\n'
                         'Current weight {winner_name} — {winner_weight}!\n'
                         'Current weight {looser_name} — {looser_weight}...\n'
                         'Arena for a best:\n'
                         '@fr0staman_chat') \
            .format(winner_name=hbold(first_names[0]),
                    looser_name=hbold(first_names[1]),
                    diff=hbold(damage_second),
                    winner_weight=hbold(hryak_sizes[0] + damage_second),
                    looser_weight=hbold(hryak_sizes[1] - damage_second)
                    )
        await HandPig.battle_win(damage_second, users_id[0], get_date())
        await HandPig.battle_loose(damage_second, users_id[1], get_date())
    elif status == 2:
        message_text = _('🎉 Victory for {winner_name}! 🎉\n'
                         '\n'
                         'He adds to himself mass {diff} kg, but {looser_name} minus...\n'
                         'Current weight {winner_name} — {winner_weight}!\n'
                         'Current weight {looser_name} — {looser_weight}...\n'
                         '\n'
                         'Run to the @fr0staman_chat') \
            .format(winner_name=hbold(first_names[1]),
                    looser_name=hbold(first_names[0]),
                    diff=hbold(damage_first),
                    winner_weight=hbold(hryak_sizes[1] + damage_first),
                    looser_weight=hbold(hryak_sizes[0] - damage_first)
                    )
        await HandPig.battle_win(damage_first, users_id[1], get_date())
        await HandPig.battle_loose(damage_first, users_id[0], get_date())
    elif status == 3:
        damage_second = hryak_sizes[1] // 3
        message_text = _('🍖 <b><i>CRITICAL DAMAGE!</i></b> 🍖\n'
                         '\n'
                         '{winner_name} melts +{diff} kg of fat, but {looser_name} - fuse it on @fr0staman_chat\n'
                         '\n'
                         'Current weight {winner_name} — {winner_weight}!\n'
                         'Current weight {looser_name} — {looser_weight}...\n'
                         ) \
            .format(winner_name=hbold(first_names[0]),
                    looser_name=hbold(first_names[1]),
                    diff=hbold(damage_second),
                    winner_weight=hbold(hryak_sizes[0] + damage_second),
                    looser_weight=hbold(hryak_sizes[1] - damage_second)
                    )

        await HandPig.battle_win(damage_second, users_id[0], get_date())
        await HandPig.battle_loose(damage_second, users_id[1], get_date())
    elif status == 4:
        damage_first = hryak_sizes[0] // 3
        message_text = _('⚡ <b><i>CRITICAL DAMAGE! УРОН!</i></b> ⚡\n'
                         '\n'
                         '{winner_name} eats to him mass +{diff} kg, but {looser_name}...\n'
                         'So pity.\n'
                         'Current weight {winner_name} — {winner_weight}!\n'
                         'Current weight {looser_name} — {looser_weight}...\n'
                         'Go to @fr0staman_chat!\n') \
            .format(winner_name=hbold(first_names[1]),
                    looser_name=hbold(first_names[0]),
                    diff=hbold(damage_first),
                    winner_weight=hbold(hryak_sizes[1] + damage_first),
                    looser_weight=hbold(hryak_sizes[0] - damage_first)
                    )
        await HandPig.battle_win(damage_first, users_id[1], get_date())
        await HandPig.battle_loose(damage_first, users_id[0], get_date())
    elif status == 5:
        damage_second = int(hryak_sizes[1] // 1.5)
        message_text = _('🔥 <b><i>KNOCKOUT!</i></b> 🔥\n'
                         '\n'
                         '{winner_name} knocking and gets +{diff} kg!\n'
                         '{looser_name} looses his meat.\n'
                         'Current weight {winner_name} — {winner_weight}!\n'
                         'Current weight {looser_name} — {looser_weight}...\n'
                         'Arena for the best:\n'
                         '@fr0staman_chat'
                         ) \
            .format(winner_name=hbold(first_names[0]),
                    looser_name=hbold(first_names[1]),
                    diff=hbold(damage_second),
                    winner_weight=hbold(hryak_sizes[0] + damage_second),
                    looser_weight=hbold(hryak_sizes[1] - damage_second)
                    )
        await HandPig.battle_win(damage_second, users_id[0], get_date())
        await HandPig.battle_loose(damage_second, users_id[1], get_date())
    elif status == 6:
        damage_first = int(hryak_sizes[0] // 1.5)
        message_text = _('🥩 <b><i>KNOCKOUT!</i></b> 🥩\n'
                         '\n'
                         '{winner_name} knocking and receives +{diff} kg!\n'
                         '{looser_name} looses his meat.\n'
                         '\n'
                         'Current weight {winner_name} — {winner_weight}!\n'
                         'Current weight {looser_name} — {looser_weight}...\n'
                         'Arena for the best:\n'
                         '@fr0staman_chat') \
            .format(
            winner_name=hbold(first_names[1]),
            looser_name=hbold(first_names[0]),
            diff=hbold(damage_first),
            winner_weight=hbold(hryak_sizes[1] + damage_first),
            looser_weight=hbold(hryak_sizes[0] - damage_first)
        )
        await HandPig.battle_win(damage_first, users_id[1], get_date())
        await HandPig.battle_loose(damage_first, users_id[0], get_date())
    else:
        final_damage = damage_first if hryak_sizes[0] > hryak_sizes[1] else damage_second

        message_text = _('🍽 <b>DRAW!</b> 🍽\n'
                         '\n'
                         'Because piggies {draw1_name} and {draw2_name} fought veeery worthy, then both get {diff} kg.') \
            .format(
            draw1_name=hbold(first_names[0]),
            draw2_name=hbold(first_names[1]),
            diff=hbold(final_damage)
        )
        await HandPig.battle_draw(final_damage, users_id[0], users_id[1], get_date())
    return EditMessageText(
        message_text,
        inline_message_id=query.inline_message_id,
        disable_web_page_preview=True
    )
