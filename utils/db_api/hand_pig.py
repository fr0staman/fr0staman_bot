import re

from aiogram.utils.markdown import hbold

from modules.base.date import get_date
from modules.base.lang import lng
from modules.hryak.consts import top_10_header
from modules.inline_mode.consts import top10_chat_header, top10_win_header
from modules.inline_mode.hryak_text import get_flag
from .storages import MySQLConnection


class HandPig(MySQLConnection):
    @staticmethod
    async def update_counter():
        sql = 'UPDATE counter SET count = count + 1;'
        await HandPig._make_request(sql)

    @staticmethod
    async def get_hryak_chat(chat_id: int):
        sql = 'SELECT f_name, user_id FROM inline_groups LEFT JOIN hryak_day ON hryak_day.ig_id = inline_groups.id ' \
              'INNER JOIN inline_users ON inline_groups.iu_id = inline_users.id' \
              'WHERE hryak_day.date = %s AND group_id = %s; '
        params = (chat_id, )
        result = await HandPig._make_request(sql, params, fetch=True, mult=True)
        if len(result) == 0:
            return -1, -1
        else:
            for value in result:
                return value['f_name'], value['user_id']

    @staticmethod
    async def add_hryak_chat(chat_id: int):
        try:
            sql = 'INSERT INTO hryak_day VALUES ((SELECT id from inline_groups WHERE group_id = %s ORDER BY rand() LIMIT 1), %s);'
            params = (chat_id, get_date())
            await HandPig._make_request(sql, params)
        except:
            return -1
        return 0

    @staticmethod
    async def update_name(user_id, name):
        sql = 'UPDATE inline_users SET name = %s WHERE user_id = %s;'
        params = (name, user_id)
        await HandPig._make_request(sql, params)

    @staticmethod
    async def get_about_user(user_id):
        sql = 'SELECT size_cm, status, f_name, name FROM inline_users WHERE user_id = %s;'
        params = (user_id,)
        sizeof = await HandPig._make_request(sql, params, fetch=True, mult=True)
        if not bool(sizeof):
            return -1, -1, -1, -1
        else:
            for value in sizeof:
                return value['size_cm'], value['status'], value['f_name'], value['name']

    @staticmethod
    async def get_hryak_size(user_id: int):
        await HandPig.update_counter()
        sql = 'SELECT size_cm, status, CAST(COALESCE(100/((win+rout)/win), 0) as UNSIGNED) as winrate, ' \
              'f_name, name FROM inline_users WHERE user_id = %s AND date = %s; '
        params = (user_id, get_date())
        sizeof = await HandPig._make_request(sql, params, fetch=True, mult=True)
        if not bool(sizeof):
            return -1, -1, -1, -1
        else:
            for value in sizeof:
                if value['name'] != '':
                    return value['size_cm'], value['status'], value['winrate'], value['name']
                else:
                    return value['size_cm'], value['status'], value['winrate'], value['f_name']

    @staticmethod
    async def insert_hryak(lang, user_id, f_name, hryak_size):
        if lang is None or len(lang) != 2:
            lang = 'en'
        if hryak_size == 0:
            hryak_size = 1
        sql = 'SELECT status FROM inline_users WHERE user_id = %s;'
        params = (user_id, )
        check = await HandPig._make_request(sql, params, fetch=True, mult=True)
        if not bool(check):
            sql = 'INSERT INTO inline_users (f_name, user_id, size_cm, date, lang) VALUES (%s, %s, %s, %s, %s);'
            params = (f_name, user_id, hryak_size, get_date(), lang)
            await HandPig._make_request(sql, params)
            status = 0
        else:
            status = 0
            status = check[0]['status']
            if status == 1:
                kg_for_status = 100
            elif status == 2:
                kg_for_status = 500
            else:
                kg_for_status = 0
            sql = 'UPDATE inline_users SET size_cm = %s + %s + COALESCE((SELECT kg_now FROM game WHERE user_id = %s ' \
                  'ORDER BY kg_now DESC LIMIT 1), 0), date = %s, gifted = 0, lang = %s WHERE user_id = %s; '
            params = (hryak_size, kg_for_status, user_id, get_date(), lang, user_id)
            await HandPig._make_request(sql, params)
        return status

    @staticmethod
    async def get_name_n_size(first_id, second_id):
        sql = 'SELECT f_name, size_cm, user_id, name, date FROM inline_users WHERE (user_id = %s OR user_id = %s); '
        params = (first_id, second_id)
        values = await HandPig._make_request(sql, params, fetch=True, mult=True)
        if len(values) == 2:
            f_names = []
            hryak_sizes = []
            user_hryak_id = []
            for value in values:
                if str(value['date']) != get_date():
                    return value['f_name'], -1, value['user_id']

                if value['name'] != '':
                    f_names.append(value['name'])
                else:
                    f_names.append(value['f_name'])
                hryak_sizes.append(int(value['size_cm']))
                user_hryak_id.append(int(value['user_id']))
            return f_names, hryak_sizes, user_hryak_id
        return 0, 0, 0

    @staticmethod
    async def get_top10_chat(lang, group_id):
        sql = ("SELECT f_name, size_cm, lang, name FROM inline_users, inline_groups "
               "WHERE date = %s and inline_users.id = inline_groups.iu_id and inline_groups.group_id = %s "
               "ORDER BY size_cm DESC LIMIT 10;")
        params = (get_date(), group_id)
        exit_value = await HandPig._make_request(sql, params, fetch=True, mult=True)
        message_text = ''
        header = hbold(top10_chat_header[lng(lang)])
        i = 0
        for value in exit_value:
            i = i + 1
            name = re.sub(r'[<>\'\"]', '', value['name'] or value['f_name'] or 'хряк')
            text_params = (hbold(i), get_flag(value['lang']), name, hbold(value['size_cm']))
            message_text = message_text + ('%s. %s %s - %s кг\n' % text_params)
        header = header + message_text
        if message_text == '':
            header = 'В вашем хлеву нет хряков...'
        return header

    @staticmethod
    async def get_top10_global(lang):
        sql = "SELECT f_name, size_cm, lang, name FROM inline_users WHERE date = %s ORDER BY size_cm DESC LIMIT 10; "
        params = (get_date(), )
        exit_value = await HandPig._make_request(sql, params, fetch=True, mult=True)
        message_text = ''
        header = hbold(top_10_header[lng(lang)])
        i = 0
        for value in exit_value:
            i = i + 1
            if value['name'] != '':
                message_text = message_text + \
                               ('%s. %s %s - %s кг\n' % (
                                   hbold(i), get_flag(value['lang']),
                                   re.sub(r'[<>\'\"]', '', value['name']) if value['name'] != '' else 'хряк',
                                   hbold(value['size_cm'])))
            else:
                message_text = message_text + \
                               ('%s. %s %s - %s кг\n' % (
                                   hbold(i), get_flag(value['lang']),
                                   re.sub(r'[<>\'\"]', '', value['f_name']) if value['f_name'] != '' else 'хряк',
                                   hbold(value['size_cm'])))
        header = header + message_text
        return header

    @staticmethod
    async def get_top10_win(lang):
        sql = "SELECT f_name, win, lang, name FROM inline_users ORDER BY win DESC LIMIT 10;"
        exit_value = await HandPig._make_request(sql, fetch=True, mult=True)
        message_text = ''
        header = hbold(top10_win_header[lng(lang)])
        i = 0
        for value in exit_value:
            i = i + 1
            name = re.sub(r'[<>\'\"]', '', value['name'] or value['f_name'] or 'хряк')
            text_params = (hbold(i), get_flag(value['lang']), name, hbold(value['win']))
            message_text = message_text + ('%s. %s %s - %s wins!\n' % text_params)
        header = header + message_text
        return header

    @staticmethod
    async def add_group_to_user(chat_id, user_id):
        sql = 'SELECT * FROM inline_groups, inline_users WHERE inline_groups.group_id = %s AND ' \
               'inline_users.user_id = %s AND iu_id = inline_users.id AND inline_users.date = %s; '
        params = (chat_id, user_id, get_date())
        check = await HandPig._make_request(sql, params, fetch=True, mult=True)
        if not bool(check):
            sql = 'INSERT INTO inline_groups (group_id, iu_id) SELECT %s, id FROM inline_users WHERE user_id = %s and date = %s; '
            params = (chat_id, user_id, get_date())
            await HandPig._make_request(sql, params)

    @staticmethod
    async def battle_win(damage, user_id, date):
        sql = 'UPDATE inline_users SET size_cm = size_cm + %s, win=win+1 WHERE user_id = %s AND date = %s;'
        params = (damage, user_id, date)
        await HandPig._make_request(sql, params)

    @staticmethod
    async def battle_loose(damage, user_id, date):
        sql = 'UPDATE inline_users SET size_cm = size_cm - %s, rout=rout+1 WHERE user_id = %s AND date = %s;'
        params = (damage, user_id, date)
        await HandPig._make_request(sql, params)

    @staticmethod
    async def battle_draw(damage, first_user, second_user, date):
        sql = 'UPDATE inline_users SET size_cm = size_cm + %s, win=win+1 WHERE (user_id = %s OR user_id = %s) AND ' \
                  'date = %s;'
        params = (damage, first_user, second_user, date)
        await HandPig._make_request(sql, params)
