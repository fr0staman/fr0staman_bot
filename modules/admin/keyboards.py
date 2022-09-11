from aiogram.types import InlineKeyboardMarkup, InlineKeyboardButton

from modules.inline_mode.callbacks import inline_callback


def check_subscribe():
    return InlineKeyboardMarkup().row(
        InlineKeyboardButton('Я подписался ☺',
                             callback_data=inline_callback.new(action='subscribe', id='0'))
    )


def check_subscribe_for_gift():
    return InlineKeyboardMarkup().row(
        InlineKeyboardButton('Получить 🎂',
                             callback_data=inline_callback.new(action='subscribe_gift', id='0'))
    )


def to_channel():
    return InlineKeyboardMarkup().row(
        InlineKeyboardButton('Получить подарок ↗', url='https://t.me/fr0staman_channel/24')
    )
