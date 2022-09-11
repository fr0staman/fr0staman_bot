from aiogram import types

from aiogram.dispatcher.webhook import SendMessage

from core.config import CREATOR_ID
from core.misc import dp, bot, _
from modules.base.keyboards import game_inline
from utils.db_api import User, Settings


@dp.message_handler(lambda m: types.ChatType.is_group_or_super_group,
                    commands=['эпик', 'epyc', 'епік'],
                    commands_prefix='!/#')
async def cmd_epyc(message):
    try:
        user_request = await bot.get_chat_member(chat_id=message.chat.id, user_id=message.from_user.id)
    except:
        return SendMessage(message.chat.id, _('Does not have information.'), reply_to_message_id=message.message_id)
    try:
        if (hasattr(user_request, 'can_restrict_members') and user_request.can_restrict_members is True) \
                or user_request.status == 'creator':
            command, tpybabemp, msg_args = message.text.partition(' ')
            if msg_args:
                option, temp, setting = msg_args.partition(' ')
                if option == 'приветствие':
                    if setting == 'выкл':
                        await Settings.set_settings_group(message.chat, 1)
                        return SendMessage(message.chat.id, _('Greetings disabled successfully!'))
                    elif setting == 'вкл':
                        await Settings.set_settings_group(message.chat, 0)
                        return SendMessage(message.chat.id, _('Greetings enabled successfully!'))
                    else:
                        return SendMessage(message.chat.id, _('This option exist, but give me a correct parameter!'))
                elif option == 'топ':
                    if setting.isdigit():
                        await Settings.top10_setting(message.chat, int(setting))
                        return SendMessage(message.chat.id,
                                           _('Successfully changed!\nNow, visible pigs in the top starts with {setting} kg.').format(setting=setting))
                    else:
                        return SendMessage(message.chat.id, _('You entered incorrect parameter.\nOnly a number.'))
                else:
                    return SendMessage(message.chat.id, _('That function not exist.'))
            else:
                return SendMessage(message.chat.id, _('EPYC.\nJust EPYC.'))
        else:
            return SendMessage(message.chat.id,
                               _('I am Valera turururu,\nI am Valera turururu,\nI am Valera turururu\n\nAnd you\'re not a admin.'))
    except Exception as e:
        await bot.send_message(CREATOR_ID, str(message) + '\n' + str(e))
        return SendMessage(message.chat.id, _('techincal chocolaps'))


@dp.message_handler(lambda m: m.chat.id == m.from_user.id, commands=['start'], commands_prefix='!/#')
async def cmd_start(message):
    await User.register_user(message.from_user)
    if (await bot.get_chat_member(-1001592533996, message.from_user.id)).status != 'left':
        await bot.send_message(message.chat.id,
                               _('You are subscribed on <a href="https://t.me/fr0staman_channel">channel</a>!You are supposed to <b>+100</b> kg <b>forever</b>.')
                               )
        await User.subscribe_user(message.from_user.id)
    else:
        await bot.send_message(message.chat.id,
                               _('Subscribe on <a href="https://t.me/fr0staman_channel">channel</a>!\nDo not waste opportunity to get <b>+100</b> kg forever.'))

    return SendMessage(message.chat.id, _("Hello!\nThis bot does nothing useful.\n\nI\'m serious."
                                          "\n\nList of commands: - /help\n"
                                          "Questions, suggestions - @fr0staman.\n\nI will be glad :)"
                                          "\n"
                                          "News - @fr0staman_channel"),
                       reply_markup=game_inline(message.from_user.language_code))


@dp.message_handler(commands=['help'], commands_prefix='!/#')
async def cmd_help(message):
    return SendMessage(message.chat.id, _("Help for the bot:\n"
                                          ""
                                          "https://telegra.ph/Pomoshch--fr0staman-bot-09-30"))
