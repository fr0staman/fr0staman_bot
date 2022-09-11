from .storages import MySQLConnection


class Inline(MySQLConnection):
    @staticmethod
    async def get_all_voices():
        sql = 'SELECT id, url, user_id, status, caption FROM `inline_voices` WHERE status = 1 ORDER BY RAND() LIMIT 30;'
        r = await Inline._make_request(sql, fetch=True, mult=True)
        return r
