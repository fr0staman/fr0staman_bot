from .storages import MySQLConnection
from aiogram import types


class User(MySQLConnection):
    @staticmethod
    async def register_user(user: types.User):
        sql = "INSERT IGNORE INTO users (user_id) VALUES (%s);"
        params = (user.id,)
        await User._make_request(sql, params)

    @staticmethod
    async def user_exists(user: types.User):
        sql = 'SELECT user_id FROM inline_users WHERE user_id = %s;'
        params = (user.id,)
        r = await User._make_request(sql, params, fetch=True, mult=True)
        return bool(r)

    @staticmethod
    async def subscribe_user(user: types.User):
        sql = 'UPDATE inline_users SET status = 1, size_cm = size_cm + 100 WHERE user_id = %s AND status != 1 AND ' \
              'status != 2; '
        params = (user.id,)
        await User._make_request(sql, params)

    @staticmethod
    async def add_500_kg(user: types.User):
        sql = 'UPDATE inline_users SET gifted = 1, size_cm = size_cm + 500 WHERE user_id = %s AND gifted = 0;'
        params = (user.id,)
        await User._make_request(sql, params)

    @staticmethod
    async def add_voice_group(url, user_id, status, caption):
        sql = 'INSERT INTO `inline_voices` (url, user_id, status, caption) VALUES (%s, %s, %s, %s);'
        params = (url, user_id, status, caption)
        await User._make_request(sql, params)

    @staticmethod
    async def add_user_kg(kg: int, user_id: int):
        sql = 'UPDATE `inline_users` SET size_cm = size_cm + %s WHERE user_id = %s;'
        params = (kg, user_id)
        await User._make_request(sql, params)

