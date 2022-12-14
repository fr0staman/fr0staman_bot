from modules.base.date import get_timestamp, get_day, get_month
from .flags import flags


def get_flag(lang):
    LANG_EXIST = lang in flags

    if LANG_EXIST:
        return flags[lang]
    return "πΊπΈ"


def get_emoji(hryak_size):
    emoji = 'π¦΄'
    if hryak_size > 10000:
        emoji = 'πͺ'
    elif hryak_size > 8000:
        emoji = 'β'
    elif hryak_size > 7000:
        emoji = 'π«'
    elif hryak_size > 6000:
        emoji = 'π '
    elif hryak_size > 5000:
        emoji = 'π'
    elif hryak_size > 4000:
        emoji = 'π'
    elif hryak_size > 3000:
        emoji = 'π₯'
    elif hryak_size > 2000:
        emoji = 'β’οΈ'
    elif hryak_size == 1488:
        emoji = 'β‘β‘'
    elif hryak_size > 1000:
        emoji = 'β£οΈ'
    elif hryak_size > 800:
        emoji = 'π·'
    elif hryak_size == 777:
        emoji = 'π°'
    elif hryak_size == 666:
        emoji = 'πΉ'
    elif hryak_size > 500:
        emoji = 'ππ¨'
    elif hryak_size > 300:
        emoji = 'π'
    elif hryak_size > 100:
        emoji = 'π½'
    elif hryak_size > 20:
        emoji = 'π·'
    elif hryak_size == 18:
        emoji = 'π'
    elif hryak_size > 10:
        emoji = 'π'
    elif hryak_size == 1:
        emoji = 'π½'
    return emoji


def calculate_hryak_size(user_id):
    category = (get_timestamp() / 5527 * get_day() / get_month() +
                user_id / (get_day() * get_month())) % 25
    if category < 0.05:
        category = 0.39
    elif category < 0.3:
        category = 1
    elif category < 6:
        category = 2
    elif category < 12:
        category = 3
    elif category < 21:
        category = 5
    elif category < 25:
        category = 7

    hryak_size = int((get_timestamp() / get_day() * get_month() / 1009 + user_id) %
                     (4049 + 10 * (get_day() + ((get_month() - 8) * 30))) / category)
    return hryak_size
