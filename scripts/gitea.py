from urllib.parse import urlunparse, urlparse
from html.parser import HTMLParser
from time import sleep
import random

from requests import Session
from requests.auth import HTTPBasicAuth
import requests

GITEA_USER = "bot"
GITEA_EMAIL = "bot@example.com"
GITEA_PASSWORD = "foobarpassword"

TOTAL_NUM_REPOS = 100
REPOS = []


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
    def __get_csrf_token(page: str) -> str:
        parser = ParseCSRFGiteaForm()
        parser.feed(page)
        csrf = parser.token
        return csrf

    def get_csrf_token(self, url: str) -> str:
        resp = self.session.get(url, allow_redirects=False)
        if resp.status_code != 200 and resp.status_code != 302:
            print(resp.status_code, resp.text)
            raise Exception(f"Can't get csrf token: {resp.status_code}")
        csrf = self.__get_csrf_token(resp.text)
        return csrf


def register(client: HTMLClient):
    url = "http://localhost:8080/user/sign_up"
    csrf = client.get_csrf_token(url)
    payload = {
        "_csrf": csrf,
        "user_name": GITEA_USER,
        "password": GITEA_PASSWORD,
        "retype": GITEA_PASSWORD,
        "email": GITEA_EMAIL,
    }
    resp = client.session.post(url, data=payload, allow_redirects=False)


def login(client: HTMLClient):
    url = "http://localhost:8080/user/login"
    csrf = client.get_csrf_token(url)
    payload = {
        "_csrf": csrf,
        "user_name": GITEA_USER,
        "password": GITEA_PASSWORD,
        "remember": "on",
    }
    resp = client.session.post(url, data=payload, allow_redirects=False)
    print(f"login {client.session.cookies}")
    if resp.status_code == 302:
        print("User logged in")
        return

    raise Exception(f"[ERROR] Authentication failed. status code {resp.status_code}")


def create_repositories(client: HTMLClient):
    print("foo")

    def get_repository_payload(csrf: str, name: str):
        data = {
            "_csrf": csrf,
            "uid": "1",
            "repo_name": name,
            "description": f"this repository is named {name}",
            "repo_template": "",
            "issue_labels": "",
            "gitignores": "",
            "license": "",
            "readme": "Default",
            "default_branch": "master",
            "trust_model": "default",
        }
        return data

    url = "http://localhost:8080/repo/create"
    for repo in REPOS:
        csrf = client.get_csrf_token(url)
        resp = client.session.post(url, data=get_repository_payload(csrf, repo))
        print(f"Created repository {repo}")
        if resp.status_code != 302 and resp.status_code != 200:
            raise Exception(
                f"Error while creating repository: {repo} {resp.status_code}"
            )
        add_tag(repo, client)


def add_tag(repo: str, client: HTMLClient):
    print("adding tags")
    tag = "testing"
    url = f"http://{GITEA_USER}:{GITEA_PASSWORD}@localhost:8080/api/v1/repos/{GITEA_USER}/{repo}/topics/{tag}"
    resp = requests.put(url)
    if resp.status_code != 204:
        print(f"Error while adding tags repository: {repo} {resp.status_code}")
        raise Exception(
            f"Error while adding tags repository: {repo} {resp.status_code}"
        )


if __name__ == "__main__":
    for i in range(TOTAL_NUM_REPOS):
        REPOS.append(f"reopsitory_{i}")
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
            login(client)
            create_repositories(client)
            break
        except Exception as e:
            print(f"Error: {e}")
            print(f"Retrying {count} time")
            count += 1
            sleep(5)
            continue
