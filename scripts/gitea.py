from urllib.parse import urlunparse, urlparse
from html.parser import HTMLParser
import concurrent.futures
import random
import aiohttp
import asyncio

# from requests import Session
#from requests.auth import HTTPBasicAuth
#import requests
import threading

GITEA_USER = "bot"
GITEA_EMAIL = "bot@example.com"
GITEA_PASSWORD = "foobarpassword"

TOTAL_NUM_REPOS = 100
REPOS = []


# async def check_online(port: int):
#    count = 0
#    while True:
#        try:
#            res = requests.get(
#                f"http://localhost:{port}/", allow_redirects=False
#            )
#            if any([res.status == 302, res.status == 200]):
#                break
#        except:
#            sleep(2)
#            print(f"[{port}] Retrying check_online {count} time")
#            count += 1
#            continue


class ParseCSRFGiteaForm(HTMLParser):
    token: str = None

    def handle_starttag(self, tag: str, attrs: (str, str)):
        if self.token:
            return

        if tag != "input":
            return

        token = None
        for index, (k, v) in enumerate(attrs):
            if k == "value":
                token = v

            if all([k == "name", v == "_csrf"]):
                if token:
                    self.token = token
                    return
                for inner_index, (nk, nv) in enumerate(attrs, start=index):
                    if nk == "value":
                        self.token = nv
                        return


class HTMLClient:
    def __init__(self, session):
        self.session = session

    @staticmethod
    def __get_csrf_token(page: str) -> str:
        parser = ParseCSRFGiteaForm()
        parser.feed(page)
        csrf = parser.token
        return csrf

    async def get_csrf_token(self, url: str) -> str:
        resp = await self.session.get(url, allow_redirects=False)
        if resp.status != 200 and resp.status != 302:
            print(resp.status, await resp.text())
            raise Exception(f"Can't get csrf token: {resp.status}")
        csrf = self.__get_csrf_token(await resp.text())
        return csrf


async def install(port: int, client: HTMLClient):
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
        "app_url": f"http://localhost:{port}/",
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
    await client.session.post(f"http://localhost:{port}", data=INSTALL_PAYLOAD)




async def register(port: int, client: HTMLClient):
    url = f"http://localhost:{port}/user/sign_up"
    csrf = await client.get_csrf_token(url)
    payload = {
        "_csrf": csrf,
        "user_name": GITEA_USER,
        "password": GITEA_PASSWORD,
        "retype": GITEA_PASSWORD,
        "email": GITEA_EMAIL,
    }
    await client.session.post(url, data=payload, allow_redirects=False)


async def login(port: int, client: HTMLClient):
    url = f"http://localhost:{port}/user/login"
    csrf =await  client.get_csrf_token(url)
    payload = {
        "_csrf": csrf,
        "user_name": GITEA_USER,
        "password": GITEA_PASSWORD,
        "remember": "on",
    }
    resp = await client.session.post(url, data=payload, allow_redirects=False)
    #print(f"login {client.session.cookies}")
    if resp.status == 302:
        print("User logged in")
        return

    raise Exception(f"[ERROR] Authentication failed. status code {resp.status}")


async def create_repositories(port: int, client: HTMLClient):
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

    url = f"http://localhost:{port}/repo/create"
    for repo in REPOS:
        csrf = await client.get_csrf_token(url)
        resp = await client.session.post(url, data=get_repository_payload(csrf, repo))
        print(f"Created repository {repo} on {url}")
        if resp.status != 302 and resp.status != 200:
            raise Exception(
                f"Error while creating repository: {repo} {resp.status}"
            )
        add_tag(port, repo, client)


async def add_tag(port: int, repo: str, client: HTMLClient):
    print("adding tags")
    tag = "testing"
    url = f"http://{GITEA_USER}:{GITEA_PASSWORD}@localhost:{port}/api/v1/repos/{GITEA_USER}/{repo}/topics/{tag}"
    resp = await client.session.put(url)
    if resp.status != 204:
        print(f"Error while adding tags repository: {repo} {resp.status}")
        raise Exception(
            f"Error while adding tags repository: {repo} {resp.status}"
        )


async def init(gitea: int):
    print(f"Initializing gitea: {gitea}")
    port = gitea + 11000
    print(f"[{gitea}] Instance online")
    print(f"[{gitea}]Instance configured and installed")
    async with aiohttp.ClientSession() as session:
        client = HTMLClient(session)
        #    check_online(port)
        await install(port, client)

        count = 0
        while True:
            try:
                await register(port, client)
                print("User registered")
                await login(port, client)
                await create_repositories(port, client)
                break
            except Exception as e:
                print(f"Error: {e}")
                print(f"Retrying {count} time")
                count += 1
                await asyncio.sleep(5)
                continue


async def init_all():
    tasks = []
    loop = asyncio.get_event_loop()
    for gitea in range(0, 100):
        tasks.append(
            loop.create_task(init(gitea))
        )
    await asyncio.gather(*tasks)


if __name__ == "__main__":
    for i in range(TOTAL_NUM_REPOS):
        REPOS.append(f"repository_{i}")

    asyncio.run(init_all())

#    threads = []
#    for gitea in range(0,100):
#        init(gitea)
#        t = threading.Thread(target=init, args=(gitea,))
#        threads.append(t)
#        t.start()
#        if len(threads) == 12:
#            for i, t in enumerate(threads):
#                print(f"waiting on {i}")
#                t.join()
#            threads = []
