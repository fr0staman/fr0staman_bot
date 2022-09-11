from aiogram import types
from aiogram.dispatcher.webhook import SendMessage
from aiogram.utils.markdown import hlink, hbold

from core.config import CREATOR_ID, CHANNEL_ID
from core.misc import bot, dp, _
from utils.db_api import Settings

from modules.admin.keyboards import check_subscribe, check_subscribe_for_gift


@dp.message_handler(lambda m: m.from_user.id == CREATOR_ID, commands=['publicate'])
async def cmd_publicate(message):
    return SendMessage(CHANNEL_ID, _('<b>Thank you!</b>\n\nYour gift :)'),
                       reply_markup=check_subscribe_for_gift())


@dp.message_handler(lambda m: any(x.id == 1718982458 for x in m.new_chat_members),
                    content_types=types.ContentType.NEW_CHAT_MEMBERS)
async def react_added_me(message):
    await Settings.register_chat(message.chat)
    return SendMessage(message.chat.id, _("Hello!\nThis bot does nothing useful.\n\nI\'m serious."
                                          "\n\nList of commands: - /help\n"
                                          "Questions, suggestions - @fr0staman.\n\nI will be glad :)"
                                          "\n"
                                          "News - @fr0staman_channel"))


@dp.message_handler(lambda m: m.from_user.id == CREATOR_ID, commands=['gift_500'])
async def cmd_gift_500(message):
    return SendMessage(CHANNEL_ID,
                       _('<b>Important!</b>\n\n'
                         'Who subscribed on <a href="https://t.me/fr0staman_channel">channel</a> '
                         '- receives +500 kg to his hang pig <b>forever</b>!\n'),
                       disable_web_page_preview=True,
                       reply_markup=check_subscribe_for_gift())


@dp.message_handler(content_types=types.ContentType.NEW_CHAT_MEMBERS)
async def react_new_member(message):
    if (await Settings.get_chat_settings(message.chat)) == 0:
        if hasattr(message, "new_chat_members"):
            user_name = message.new_chat_members[0].first_name
            user_id = message.new_chat_members[0].id
        else:
            user_name = message.from_user.first_name
            user_id = message.from_user.id
        chat_title = message.chat.title
        mention = hlink(user_name, f'tg://user?id={str(user_id)}')

        return SendMessage(message.chat.id,
                           _('Hru, {mention}!\n'
                             'Welcome to the {chat_title}!\n\n'
                             'Start chat game  — /grow\n'
                             'Help for the bot — /help\n\n'
                             'Bot news — @fr0staman_channel')
                           .format(mention=mention, chat_title=hbold(chat_title)),
                           reply_to_message_id=message.message_id)


@dp.message_handler(content_types=types.ContentType.LEFT_CHAT_MEMBER)
async def react_leave_member(message):
    if (await Settings.get_chat_settings(message.chat)) == 0:
        if hasattr(message, "left_chat_member"):
            user_name = message.left_chat_member.first_name
            user_id = message.left_chat_member.id
        else:
            user_name = message.from_user.first_name
            user_id = message.from_user.id
        chat_title = message.chat.title
        mention = hlink(user_name, f'tg://user?id={str(user_id)}')

        return SendMessage(message.chat.id,
                           _('Why are you doing this...\n\n{chat_title} lost {mention}.')
                           .format(mention=mention, chat_title=hbold(chat_title)),
                           reply_to_message_id=message.message_id)


@dp.message_handler(content_types=types.ContentType.VOICE_CHAT_STARTED)
async def react_vc_started(message):
    if (await Settings.get_chat_settings(message.chat)) == 0:
        return SendMessage(message.chat.id, _('Co-co-co, fucking cock voice chat, co-co-co!'),
                           reply_to_message_id=message.message_id)
