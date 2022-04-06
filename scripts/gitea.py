from urllib.parse import urlunparse, urlparse
from html.parser import HTMLParser
from time import sleep

from requests import Session
import requests

GITEA_USER = "bot"
GITEA_EMAIL = "bot@example.com"
GITEA_PASSWORD = "foobarpassword"

requests.get


def check_online():
    count = 0
    while True:
        try:
            res = requests.get(
                "http://localhost:8080/api/v1/nodeinfo", allow_redirects=False
            )
            if any([res.status_code == 302, res.status_code == 200]):
                break
        except:
            sleep(2)
            print(f"Retrying {count} time")
            count += 1
            continue


def install():
    INSTALL_PAYLOAD = {
        "db_type": "sqlite3",
        "db_host": "localhost:3306",
        "db_user": "root",
        "db_passwd": "",
        "db_name": "gitea",
        "ssl_mode": "disable",
        "db_schema": "",
        "charset": "utf8",
        "db_path": "/data/gitea/gitea.db",
        "app_name": "Gitea:+Git+with+a+cup+of+tea",
        "repo_root_path": "/data/git/repositories",
        "lfs_root_path": "/data/git/lfs",
        "run_user": "git",
        "domain": "localhost",
        "ssh_port": "2221",
        "http_port": "3000",
        "app_url": "http://localhost:8080/",
        "log_root_path": "/data/gitea/log",
        "smtp_host": "",
        "smtp_from": "",
        "smtp_user": "",
        "smtp_passwd": "",
        "enable_federated_avatar": "on",
        "enable_open_id_sign_in": "on",
        "enable_open_id_sign_up": "on",
        "default_allow_create_organization": "on",
        "default_enable_timetracking": "on",
        "no_reply_address": "noreply.localhost",
        "password_algorithm": "pbkdf2",
        "admin_name": "",
        "admin_passwd": "",
        "admin_confirm_passwd": "",
        "admin_email": "",
    }
    requests.post(f"http://localhost:8080", data=INSTALL_PAYLOAD)


class ParseCSRFGiteaForm(HTMLParser):
    token: str = None

    def handle_starttag(self, tag: str, attrs: (str, str)):
        if self.token:
            return

        if tag != "input":
            return

        token = None
        for (index, (k, v)) in enumerate(attrs):
            if k == "value":
                token = v

            if all([k == "name", v == "_csrf"]):
                if token:
                    self.token = token
                    return
                for (inner_index, (nk, nv)) in enumerate(attrs, start=index):
                    if nk == "value":
                        self.token = nv
                        return


class HTMLClient:
    session: Session

    def __init__(self):
        self.session = Session()

    @staticmethod
    def get_csrf_token(page: str) -> str:
        parser = ParseCSRFGiteaForm()
        parser.feed(page)
        csrf = parser.token
        return csrf

def register(client: HTMLClient):
    url = "http://localhost:8080/user/sign_up"
    resp = client.session.get(url, allow_redirects=False)
    if resp.status_code != 200:
        print(resp.status_code, resp.text)
        raise Exception(resp.status_code)
    csrf = client.get_csrf_token(resp.text)
    payload = {
        "_csrf": csrf,
        "user_name": GITEA_USER,
        "password": GITEA_PASSWORD,
        "retype": GITEA_PASSWORD,
        "email": GITEA_EMAIL,
    }
    resp = client.session.post(url, data=payload, allow_redirects=False)

if __name__ == "__main__":
    check_online()
    print("Instace online")
    install()
    print("Instace configured and installed")
    client = HTMLClient()
    count = 0
    while True:
        try:
            register(client)
            print("User registered")
            break
        except:
                print(f"Retrying {count} time")
                count += 1
                continue
