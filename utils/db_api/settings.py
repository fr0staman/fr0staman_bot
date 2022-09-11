from typing import Union
from aiogram import types

from .storages import MySQLConnection


class Settings(MySQLConnection):
    @staticmethod
    async def register_chat(chat: types.Chat):
        sql = 'INSERT IGNORE INTO `groups` (group_id) VALUES (%s);'
        params = (chat.id, )
        await Settings._make_request(sql, params)

    @staticmethod
    async def get_chat_settings(chat: types.Chat) -> int:
        sql = "SELECT settings FROM `groups` WHERE group_id = %s;"
        params = (chat.id, )
        r = await Settings._make_request(sql, params, fetch=True, mult=True)
        if len(r) > 0:
            if hasattr(r[0], 'settings'):
                return r[0]['settings']

    @staticmethod
    async def top10_setting(chat: types.Chat, setting):
        sql = 'UPDATE `groups` SET top10_setting = %s WHERE group_id = %s;'
        params = (setting, chat.id)
        await Settings._make_request(sql, params)

    @staticmethod
    async def set_settings_group(chat: types.Chat, setting: int):
        sql = "UPDATE `groups` SET settings = %s WHERE group_id = %s;"
        params = (setting, chat.id)
        await Settings._make_request(sql, params)
