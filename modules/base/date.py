from datetime import datetime, timezone, timedelta


def get_day():
    return datetime.now(timezone(timedelta(hours=3))).day


def get_month():
    return datetime.now(timezone(timedelta(hours=3))).month


def get_date():
    day = get_day()
    month = get_month()
    if int(day) < 10:
        day = f"0{day}"
    if int(month) < 10:
        month = f"0{month}"

    final_date = f'{datetime.now().year}-{month}-{day}'

    return final_date


def get_timediff():
    d1 = datetime(year=datetime.now().year,
                  month=datetime.now().month,
                  day=datetime.now().day,
                  hour=0,
                  minute=0
                  ) \
        .replace(tzinfo=timezone(timedelta(hours=3)))
    d2 = datetime.now(timezone(timedelta(hours=3)))
    diff = d1 - d2

    timediff = {
        'hours': diff.seconds // 3600,
        'minutes': ((diff.seconds // 60) % 60) + 1
    }
    return timediff


def get_timestamp():
    dt = datetime(year=datetime.now().year,
                  month=get_month(),
                  day=get_day(),
                  hour=12,
                  minute=36)
    timestamp = dt.replace(tzinfo=timezone.utc).timestamp()
    return timestamp
