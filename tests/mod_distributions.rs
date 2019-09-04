use bookwerx_core_rust::routes as R;
use rocket::local::Client;
use rocket::http::ContentType;
use rocket::http::Status;

// These test include testing the correct number of distributions for_account and for_tx
pub fn distributions(client: &Client, apikey: &String, accounts: &Vec<R::Account>, transactions: &Vec<R::Transaction>) -> Vec<R::Distribution> {

    let account_id1: u32 = (*accounts.get(0).unwrap()).id;
    let account_id2: u32 = (*accounts.get(1).unwrap()).id;
    let transaction_id: u32 = (*transactions.get(0).unwrap()).id;

    // 1. GET /distributions/for_tx. sb 200, empty array
    let mut response = client.get(format!("/distributions/for_tx?apikey={}&transaction_id={}", &apikey, &transaction_id)).dispatch();
    let v: Vec<R::DistributionJoined> = serde_json::from_str(&(response.body_string().unwrap())[..]).unwrap();
    assert_eq!(response.status(), Status::Ok);
    assert_eq!(v.len(), 0);

    // 1.1 GET /distributions/for_account. sb 200, empty array
    let mut response = client.get(format!("/distributions/for_account?apikey={}&account_id={}", &apikey, &account_id1)).dispatch();
    let v: Vec<R::DistributionJoined> = serde_json::from_str(&(response.body_string().unwrap())[..]).unwrap();
    assert_eq!(response.status(), Status::Ok);
    assert_eq!(v.len(), 0);

    // 2. Try to post a new distribution

    // 2.1. But first post using a non-existent apikey. 200 and db error.
    response = client.post("/distributions")
        .body(format!("apikey=notarealkey&transaction_id={}&account_id={}&amount=12550&amount_exp=2", transaction_id, account_id1))
        .header(ContentType::Form)
        .dispatch();
    let r: R::ApiError = serde_json::from_str(&(response.body_string().unwrap())[..]).unwrap();
    assert_eq!(response.status(), Status::Ok);
    assert!(r.error.len() > 0);

    // 2.2 Successful post. 200 and InsertSuccess.
    response = client.post("/distributions")
        .body(format!("&apikey={}&transaction_id={}&account_id={}&amount=12550&amount_exp=2", apikey, transaction_id, account_id1))
        .header(ContentType::Form)
        .dispatch();
    let li: R::InsertSuccess = serde_json::from_str(&(response.body_string().unwrap())[..]).unwrap();
    assert_eq!(response.status(), Status::Ok);
    assert!(li.data.last_insert_id > 0);
    // {"data":{"last_insert_id":"54"}}

    // 2.3 Successful put. 200 and UpdateSuccess
    response = client.put("/distributions")
        .body(format!("&apikey={}&id={}&account_id={}&transaction_id={}&amount=12550&amount_exp=2", apikey, li.data.last_insert_id, account_id1, transaction_id))
        .header(ContentType::Form)
        .dispatch();
    let r: R::UpdateSuccess = serde_json::from_str(&(response.body_string().unwrap())[..]).unwrap();
    assert_eq!(response.status(), Status::Ok);
    assert!(r.data.info.len() > 0);
    // {"data":{"info":"(Rows matched: 1  Changed: 0  Warnings: 0"}}

    // 3. Now verify that there's a single distribution for_tx
    response = client.get(format!("/distributions/for_tx?apikey={}&transaction_id={}", &apikey, &transaction_id)).dispatch();
    let v: Vec<R::DistributionJoined> = serde_json::from_str(&(response.body_string().unwrap())[..]).unwrap();
    assert_eq!(response.status(), Status::Ok);
    assert_eq!(v.len(), 1);

    // 3.1 Now verify that there's a single distribution for_account
    response = client.get(format!("/distributions/for_account?apikey={}&account_id={}", &apikey, &account_id1)).dispatch();
    let v: Vec<R::DistributionJoined> = serde_json::from_str(&(response.body_string().unwrap())[..]).unwrap();
    assert_eq!(response.status(), Status::Ok);
    assert_eq!(v.len(), 1);


    // 4. Post bad record...

    // 5. Now submit a single GET of the prior POST. 200 and distribution.
    // Don't do this.  Nobody cares.

    // 6. Make the 2nd Successful post. 200 and InsertSuccess
    response = client.post("/distributions")
        .body(format!("&apikey={}&transaction_id={}&account_id={}&amount=-12550&amount_exp=2", apikey, transaction_id, account_id2))
        .header(ContentType::Form)
        .dispatch();
    let li: R::InsertSuccess = serde_json::from_str(&(response.body_string().unwrap())[..]).unwrap();
    assert_eq!(response.status(), Status::Ok);
    assert!(li.data.last_insert_id > 0);
    // {"data":{"last_insert_id":"54"}}

    // 6.1 Now verify the correct count of transactions for_tx and for_account
    response = client.get(format!("/distributions/for_tx?apikey={}&transaction_id={}", &apikey, &transaction_id)).dispatch();
    let v: Vec<R::DistributionJoined> = serde_json::from_str(&(response.body_string().unwrap())[..]).unwrap();
    assert_eq!(response.status(), Status::Ok);
    assert_eq!(v.len(), 2);

    response = client.get(format!("/distributions/for_account?apikey={}&account_id={}", &apikey, &account_id1)).dispatch();
    let v: Vec<R::DistributionJoined> = serde_json::from_str(&(response.body_string().unwrap())[..]).unwrap();
    assert_eq!(response.status(), Status::Ok);
    assert_eq!(v.len(), 1);

    // 7. Retrieve _all_ distributions because we'll need to delete 'em later
    response = client.get(format!("/distributions?apikey={}", &apikey)).dispatch();
    let v: Vec<R::Distribution> = serde_json::from_str(&(response.body_string().unwrap())[..]).unwrap();
    assert_eq!(response.status(), Status::Ok);
    assert_eq!(v.len(), 2);
    v
}