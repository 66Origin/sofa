use std::collections::HashMap;
use std::time::Duration;
use failure::Error;
use serde_json::from_reader;

use reqwest::{self, Url, Method, RequestBuilder, StatusCode};

use ::database::*;
use ::types::*;
use ::error::SofaError;

/// Client handles the URI manipulation logic and the HTTP calls to the CouchDB REST API.
/// It is also responsible for the creation/access/destruction of databases.
#[derive(Debug, Clone)]
pub struct Client {
    _client: reqwest::Client,
    dbs: Vec<&'static str>,
    _gzip: bool,
    _timeout: u8,
    pub uri: String,
    pub db_prefix: String
}

impl Client {
    pub fn new<S: Into<String>>(uri: S) -> Result<Client, Error> {
        let client = reqwest::Client::builder()
            .gzip(true)
            .timeout(Duration::new(4, 0))
            .build()?;

        Ok(Client {
            _client: client,
            uri: uri.into(),
            _gzip: true,
            _timeout: 4,
            dbs: Vec::new(),
            db_prefix: String::new()
        })
    }

    fn create_client(&self) -> Result<reqwest::Client, Error> {
        let client = reqwest::Client::builder()
            .gzip(self._gzip)
            .timeout(Duration::new(self._timeout as u64, 0))
            .build()?;

        Ok(client)
    }

    pub fn get_self(&mut self) -> &mut Self {
        self
    }

    pub fn set_uri<S: Into<String>>(&mut self, uri: S) -> &Self {
        self.uri = uri.into();
        self
    }

    pub fn set_prefix<S: Into<String>>(&mut self, prefix: S) -> &Self {
        self.db_prefix = prefix.into();
        self
    }

    pub fn gzip(&mut self, enabled: bool) -> Result<&Self, Error> {
        self._gzip = enabled;
        self._client = self.create_client()?;

        Ok(self)
    }

    pub fn timeout<U: Into<u8>>(&mut self, to: U) -> Result<&Self, Error> {
        self._timeout = to.into();
        self._client = self.create_client()?;

        Ok(self)
    }

    pub fn list_dbs(&self) -> Result<Vec<String>, Error> {
        let mut response = self.get(String::from("/_all_dbs"), None)?.send()?;
        let data = response.json::<Vec<String>>()?;

        Ok(data)
    }

    fn build_dbname<S: AsRef<str>>(&self, dbname: S) -> String {
        format!("{}{}", self.db_prefix, dbname.as_ref())
    }

    pub fn db<S: AsRef<str>>(&self, dbname: S) -> Result<Database, Error> {
        let name = self.build_dbname(&dbname);

        let db = Database::new(name.clone(), self.clone());

        let path = self.create_path(name, None)?;

        let head_response = self._client.head(&path)
            .header(reqwest::header::ContentType::json())
            .send()?;

        match head_response.status() {
            StatusCode::Ok => Ok(db),
            _ => self.make_db(&dbname),
        }
    }

    pub fn make_db<S: AsRef<str>>(&self, dbname: S) -> Result<Database, Error> {
        let name = self.build_dbname(&dbname);

        let db = Database::new(name.clone(), self.clone());

        let path = self.create_path(name, None)?;

        let put_response = self._client.put(&path)
            .header(reqwest::header::ContentType::json())
            .send()?;

        let s: CouchResponse = from_reader(put_response)?;

        match s.ok {
            Some(true) => Ok(db),
            Some(false) | _ => {
                let err = s.error.unwrap_or(s!("unspecified error"));
                Err(SofaError(err).into())
            },
        }
    }

    pub fn destroy_db<S: AsRef<str>>(&self, dbname: S) -> Result<bool, Error> {
        let path = self.create_path(self.build_dbname(dbname), None)?;
        let response = self._client.delete(&path)
            .header(reqwest::header::ContentType::json())
            .send()?;

        let s: CouchResponse = from_reader(response)?;

        Ok(s.ok.unwrap_or(false))
    }

    pub fn check_status(&self) -> Result<CouchStatus, Error> {
        let response = self._client.get(&self.uri)
            .header(reqwest::header::ContentType::json())
            .send()?;

        let status = from_reader(response)?;

        Ok(status)
    }

    fn create_path<S: AsRef<str>>(&self,
        path: S,
        args: Option<HashMap<String, String>>
    ) -> Result<String, Error> {
        let mut uri = Url::parse(&self.uri)?.join(path.as_ref())?;

        if let Some(ref map) = args {
            let mut qp = uri.query_pairs_mut();
            for (k, v) in map {
                qp.append_pair(k, v);
            }
        }

        Ok(uri.into_string())
    }

    pub fn req<S: AsRef<str>>(&self,
        method: Method,
        path: S,
        opts: Option<HashMap<String, String>>
    ) -> Result<RequestBuilder, Error> {
        let uri = self.create_path(path, opts)?;
        let mut req = self._client.request(method, &uri);
        req.header(reqwest::header::Referer::new(uri.clone()));
        req.header(reqwest::header::ContentType::json());

        Ok(req)
    }

    pub fn get<S: AsRef<str>>(&self, path: S, args: Option<HashMap<String, String>>) -> Result<RequestBuilder, Error> {
        Ok(self.req(Method::Get, path, args)?)
    }

    pub fn post<S: AsRef<str>>(&self, path: S, body: String) -> Result<RequestBuilder, Error> {
        let mut req = self.req(Method::Post, path, None)?;
        req.body(body);
        Ok(req)
    }

    pub fn put<S: AsRef<str>>(&self, path: S, body: String) -> Result<RequestBuilder, Error> {
        let mut req = self.req(Method::Put, path, None)?;
        req.body(body);
        Ok(req)
    }

    pub fn head<S: AsRef<str>>(&self, path: S, args: Option<HashMap<String, String>>) -> Result<RequestBuilder, Error> {
        Ok(self.req(Method::Head, path, args)?)
    }

    pub fn delete<S: AsRef<str>>(&self, path: S, args: Option<HashMap<String, String>>) -> Result<RequestBuilder, Error> {
        Ok(self.req(Method::Delete, path, args)?)
    }
}
