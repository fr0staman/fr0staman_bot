from aiogram.types import InlineKeyboardMarkup, InlineKeyboardButton

from modules.base.lang import lng
from modules.inline_mode.consts import inline_callback, write_result, top10_world_btn, top10_chat_btn, hryak_duel, \
    top10_win_btn, inline_name_give_button, inline_callback_name, inline_hryak_of_the_day_btn


def top_10_global_private(lang):
    return InlineKeyboardMarkup().row(
        InlineKeyboardButton(top10_world_btn[lng(lang)],
                             callback_data=inline_callback.new(action='top10_global_private', id='0'))
    )


def top_10_win_private(lang):
    return InlineKeyboardMarkup().row(
        InlineKeyboardButton(top10_win_btn[lng(lang)],
                             callback_data=inline_callback.new(action='top10_win_private', id='0'))
    )


def top_10_create(user_id, lang='ru'):
    return InlineKeyboardMarkup().row(
        InlineKeyboardButton(write_result[lng(lang)],
                             callback_data=inline_callback.new(action='add_chat', id=str(user_id)))
    )


def top_10_global(lang='ru'):
    return InlineKeyboardMarkup().row(
        InlineKeyboardButton(top10_world_btn[lng(lang)],
                             callback_data=inline_callback.new(action='top10_global', id='0'))
    )


def top_10_chat(user_id, lang='ru'):
    return InlineKeyboardMarkup().row(
        InlineKeyboardButton(top10_chat_btn[lng(lang)],
                             callback_data=inline_callback.new(action='top10_chat', id=str(user_id)))
    )


def top_10_win(lang='ru'):
    return InlineKeyboardMarkup().row(
        InlineKeyboardButton(top10_win_btn[lng(lang)],
                             callback_data=inline_callback.new(action='top10_win', id='0'))
    )


def hryak_battle_markup(user_id, lang='ru'):
    return InlineKeyboardMarkup().row(
        InlineKeyboardButton(hryak_duel[lng(lang)],
                             callback_data=inline_callback.new(action='start_duel', id=str(user_id)))
    )


def give_name_markup(user_id, lang='ru', name='хряк'):
    return InlineKeyboardMarkup().row(
        InlineKeyboardButton(inline_name_give_button[lng(lang)],
                             callback_data=inline_callback_name.new(action='give_name', id=str(user_id), name=name))
    )


def inline_hryak_of_the_day_markup(lang):
    return InlineKeyboardMarkup().row(
        InlineKeyboardButton(inline_hryak_of_the_day_btn[lng(lang)],
                             callback_data=inline_callback.new(action='find_hryak', id='0'))
    )
