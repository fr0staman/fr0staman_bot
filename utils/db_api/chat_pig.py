from typing import Union

from aiogram import types

from .storages import MySQLConnection


class ChatPig(MySQLConnection):
    @staticmethod
    async def set_hryak_name(params):
        query = 'UPDATE game SET name = %s, f_name = %s WHERE group_id = %s AND user_id = %s;'
        r = await ChatPig._make_request(query, params)
        return r

    @staticmethod
    async def get_hryak_name(params):
        query = "SELECT name FROM game WHERE group_id = %s AND user_id = %s;"
        r = await ChatPig._make_request(query, params, fetch=True, mult=True)
        return r

    @staticmethod
    async def get_hryak_chrs(params):
        query = "SELECT kg_now, name FROM game WHERE group_id = %s AND user_id = %s;"
        r = await ChatPig._make_request(query, params, fetch=True, mult=True)
        return r

    @staticmethod
    async def get_top10_setting(params):
        query = 'SELECT top10_setting FROM `groups` WHERE group_id = %s;'
        r = await ChatPig._make_request(query, params, fetch=True, mult=True)
        return r

    @staticmethod
    async def get_top10_chat_nr(params):
        query = ("SELECT kg_now, name, (SELECT COUNT(*) FROM game WHERE group_id = %s) as mycount FROM game "
                 "WHERE group_id = %s ORDER BY kg_now DESC LIMIT %s, 50;")
        r = await ChatPig._make_request(query, params, fetch=True, mult=True)
        return r

    @staticmethod
    async def get_top10_chat_st(params):
        query = ("SELECT kg_now, name, (SELECT COUNT(*) FROM game WHERE group_id = %s AND kg_now > %s) as mycount FROM game"
                 " WHERE group_id = %s AND kg_now > %s ORDER BY kg_now DESC LIMIT %s, 50;")
        r = await ChatPig._make_request(query, params, fetch=True, mult=True)
        return r

    @staticmethod
    async def set_inline_name(params):
        query = 'UPDATE inline_users SET name = %s, f_name = %s WHERE user_id = %s;'
        await ChatPig._make_request(query, params)

    @staticmethod
    async def get_inline_name(params):
        query = "SELECT name FROM inline_users WHERE user_id = %s;"
        r = await ChatPig._make_request(query, params, fetch=True, mult=True)
        return r

    @staticmethod
    async def check_user_day(params):
        query = "SELECT user_id FROM game WHERE group_id = %s AND user_id = %s AND date = %s;"
        params = params
        r = await ChatPig._make_request(query, params, fetch=True, mult=True)
        return r

    @staticmethod
    async def get_hryak_values(params):
        query = "SELECT kg_now, name FROM game WHERE group_id = %s AND user_id = %s;"
        params = params
        r = await ChatPig._make_request(query, params, fetch=True, mult=True)
        return r

    @staticmethod
    async def add_new_hryak(params):
        query = "INSERT INTO game (group_id, user_id, kg_now, date, f_name) VALUES (%s, %s, %s, %s, %s);"
        params = params
        await ChatPig._make_request(query, params)

    @staticmethod
    async def update_hryak(params):
        query = "UPDATE game SET kg_now = %s, date = %s, f_name = %s WHERE group_id = %s AND user_id = %s;"
        params = params
        await ChatPig._make_request(query, params)

