from core import misc


def load_modules():
    misc.loader.load_packages(f"modules.{item}" for item in [
        'inline_mode',
        'admin',
        'hryak',
        'base',
        'debug',
        'text',
        'other',
        'media',
    ])
