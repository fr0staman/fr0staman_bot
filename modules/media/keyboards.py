from aiogram.types import InlineKeyboardMarkup, InlineKeyboardButton

from modules.media.callbacks import voice_data


def voice_reply_markup(user_id):
    return InlineKeyboardMarkup().row(
        InlineKeyboardButton('Принять ✅', callback_data=voice_data.new(action='allow_voice', id=str(user_id))),
        InlineKeyboardButton('Отставить ❌', callback_data=voice_data.new(action='disallow_voice', id=str(user_id)))
    )


def dont_allowed_voice():
    return InlineKeyboardMarkup().row(
        InlineKeyboardButton('Не принято.')
    )


def allowed_voice():
    return InlineKeyboardMarkup().row(
        InlineKeyboardButton('Принято!')
    )
