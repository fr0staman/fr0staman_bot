from aiogram.utils.exceptions import MessageNotModified

from core.config import CREATOR_ID, LOG_GROUP
from core.misc import dp
from aiogram.dispatcher.webhook import AnswerInlineQuery, SendMessage
from modules.inline_mode.inline_query import oc_inline, hryak_inline, hryak_name_inline, hryak_name_inline_give, \
    hryak_of_the_day, hryak_voice_hru

KEYWORDS = {
    "разгон": oc_inline,
    "розгін": oc_inline,
    "ос": oc_inline,
    "oc": oc_inline,

    "name": hryak_name_inline,
    "имя": hryak_name_inline,
    "імя": hryak_name_inline,
    "ім\'я": hryak_name_inline,

    "хохол": hryak_of_the_day,
    "hryak": hryak_of_the_day,
    "хряк": hryak_of_the_day,
    "cвинья": hryak_of_the_day,
    "свиня": hryak_of_the_day,

    "хрю": hryak_voice_hru,
    "хрюкни": hryak_voice_hru,
    "hru": hryak_voice_hru,
    "grunt": hryak_voice_hru,
}

COMMANDS = {
    "name": hryak_name_inline_give,
    "имя": hryak_name_inline_give,
    "імя": hryak_name_inline_give,
    "ім\'я": hryak_name_inline_give,
}


@dp.inline_handler()
async def query_text(query):
    try:
        keyword = query.query.lower()
        if keyword in KEYWORDS:
            return await KEYWORDS[keyword](query)

        command, _t, qu_arg = query.query.partition(' ')
        lowered_command = command.lower()

        if lowered_command in COMMANDS:
            return await COMMANDS[command](query, qu_arg)

        return await hryak_inline(query)
    except Exception as e:
        return SendMessage(CREATOR_ID, str(e))


@dp.errors_handler(exception=MessageNotModified)
async def message_not_modified_handler(update, error):
    return


@dp.errors_handler()
async def error_handler(update, error):
    print(error)
    return SendMessage(LOG_GROUP, str(update) + '\n\n' + str(error))
