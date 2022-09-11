from .storages import MySQLConnection
from aiogram import types


class Tech(MySQLConnection):
    @staticmethod
    async def delete_user(user: types.User):
        sql = 'DELETE FROM users WHERE user_id = %s;'
        params = (user.id, )
        await Tech._make_request(sql, params)

    @staticmethod
    async def get_all_groups():
        sql = 'SELECT group_id FROM `groups`;'
        r = await Tech._make_request(sql, fetch=True, mult=True)
        return r

    @staticmethod
    async def get_all_users():
        sql = 'SELECT user_id FROM `users`;'
        r = await Tech._make_request(sql, fetch=True, mult=True)
        return r

    @staticmethod
    async def check_group(chat: types.Chat) -> bool:
        sql = 'SELECT group_id FROM `groups` WHERE group_id = %s;'
        params = (chat.id, )
        r = await Tech._make_request(sql, params, fetch=True)
        return bool(r)

    @staticmethod
    async def register_group(chat: types.Chat):
        chat_exists = await Tech.check_group(chat)
        if chat_exists:
            sql = 'INSERT INTO `groups` (group_id) VALUES (%s);'
            params = (chat.id,)
            await Tech._make_request(sql, params)
