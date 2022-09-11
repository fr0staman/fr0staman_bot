import os
import ssl
from pathlib import Path
from dotenv import load_dotenv

load_dotenv()


def get_env(key, default=None, converter=None):
    value = os.environ.get(key, default) or default
    if converter:
        return converter(value)
    return str(value)


WEBAPP_HOST = get_env("WEBAPP_HOST")
WEBAPP_PORT = get_env("WEBAPP_PORT")

WEBHOOK_URL = get_env("WEBHOOK_URL")
WEBHOOK_PATH = get_env("WEBHOOK_PATH")
BOTAPI_HOST = get_env("BOTAPI_HOST")

SSL_KEY = get_env("SSL_KEY")
SSL_PEM = get_env("SSL_PEM")

BASE_DIR = Path(__file__).parent.parent
LOCALES_DIR = BASE_DIR / 'locales'
I18N_DOMAIN = 'fr0staman_bot'

webhook_server = {
    'webhook_path': WEBHOOK_PATH,
    'host': WEBAPP_HOST,
    'port': WEBAPP_PORT,
}

if SSL_PEM and SSL_KEY:
    context = ssl.SSLContext(ssl.PROTOCOL_TLSv1_2)
    try:
        context.load_cert_chain(SSL_PEM, SSL_KEY)
        webhook_server['ssl_context'] = context
    except Exception as e:
        print(e)

USE_WEBHOOK = get_env("USE_WEBHOOK", False) == "True"
SKIP_UPDATES = get_env("SKIP_UPDATES", False) == "True"

mysql_info = {
    'host':       get_env('DB_HOST'),
    'db':         get_env('DB_NAME'),
    'user':       get_env('DB_USER'),
    'password':   get_env('DB_PASS'),
    # Temporary
    'autocommit': True
}

CREATOR_ID = get_env("CREATOR_ID", None, int)
LOG_GROUP = get_env("LOG_GROUP", None, int)
CHANNEL_ID = get_env("CHANNEL_ID", None, int)
CONTENT_CHANNEL_ID = get_env("CONTENT_CHANNEL_ID", None, int)

BOT_TOKEN = get_env("BOT_TOKEN")
