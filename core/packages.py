import importlib
import pkgutil

REQUIREMENT_SEPARATOR = '::'


class PackagesLoader:
    def __init__(self, *, debug=False):
        self._debug = debug

        self.modules = {}

    def load_package(self, package, recursive=True):
        """
        Import all submodules of a module, recursively, including subpackages
        :param package: package (name or actual module)
        :type package: str | module
        :param recursive: recursive import
        :type recursive: :obj:`bool`
        :rtype: dict[str, types.ModuleType]
        """
        if isinstance(package, str):
            package = importlib.import_module(package)

        full_name = package.__name__
        results = {full_name: package}

        for pkg_loader, name, is_pkg in pkgutil.walk_packages(package.__path__):
            full_name = package.__name__ + '.' + name
            module = self.modules[full_name] = results[full_name] = importlib.import_module(full_name)
            if hasattr(module, 'includeme') and callable(module.includeme):
                module.includeme()

            if recursive and is_pkg:
                results.update(self.load_package(full_name))
        return results

    def load_packages(self, packages, recursive=True):
        """
        Load list of the packages
        :param packages:
        :param recursive:
        :return:
        """
        result = []
        for package in packages:
            result.append(self.load_package(package, recursive))
        return result
