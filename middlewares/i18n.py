from typing import Tuple, Any, Optional

from aiogram import types
from aiogram.contrib.middlewares.i18n import I18nMiddleware

from core.config import I18N_DOMAIN, LOCALES_DIR

ALLOWED_LANGS = ["uk", "en", "ru"]


class PIGMiddleware(I18nMiddleware):
    async def get_user_locale(self, action: str, args: Tuple[Any]) -> Optional[str]:
        user = types.User.get_current()
        lang_exist = user.language_code in ALLOWED_LANGS
        return user.language_code if lang_exist else "ru"


def setup_middleware(dp):
    i18n = PIGMiddleware(I18N_DOMAIN, LOCALES_DIR)
    dp.middleware.setup(i18n)
    return i18n
