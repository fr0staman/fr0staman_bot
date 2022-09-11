from aiogram.types import InlineKeyboardButton, InlineKeyboardMarkup

from modules.base.lang import lng
from modules.hryak.consts import bot_add_group


def game_inline(lang):
    return InlineKeyboardMarkup().row(
        InlineKeyboardButton(bot_add_group[lng(lang)], url='https://t.me/fr0staman_bot?startgroup=botstart')
    )
