
# Google Sheets Connector
An official Fluvio connector that sinks data from a Fluvio topic to a Google Sheets spreadsheet.

## Sink Connector
This connector reads records from a Fluvio topic and writes them into a Google Sheets spreadsheet, using Google’s OAuth2 service account credentials for authentication.

### Configuration
| Option              | Default  | Type           | Description                                                                                                           |
|:--------------------|:---------|:---------------|:----------------------------------------------------------------------------------------------------------------------|
| google_private_key   | -        | SecretString   | The private key from the Google service account used for authenticating API requests.                                 |
| google_client_email  | -        | SecretString   | The client email from the Google service account.                                                                     |
| google_token_url     | -        | SecretString   | The URL for obtaining OAuth2 tokens (usually `https://oauth2.googleapis.com/token`).                                  |
| spreadsheet_id       | -        | String         | The ID of the target Google Spreadsheet.                                                                              |
| topic               | -        | String         | The Fluvio topic from which the connector will consume events.                                                        |
| range               | A1       | String         | The range in the Google Sheet where data will be inserted.                                                            |
| major_dimension      | ROWS     | String         | The major dimension for writing data (`ROWS` or `COLUMNS`).                                                           |

`google_private_key`, `google_client_email`, and `google_token_url` can be set as a raw string value:
```yaml
google_private_key: "your-private-key"
google_client_email: "your-client-email"
google_token_url: "https://oauth2.googleapis.com/token"
```

Or, as a reference to secrets:
```yaml
google_private_key:
  secret:
    name: "GOOGLE_PRIVATE_KEY"
google_client_email:
  secret:
    name: "GOOGLE_CLIENT_EMAIL"
google_token_url:
  secret:
    name: "GOOGLE_TOKEN_URI"
```

#### Record Type Output
The payload sent to the Google Sheet is a JSON serialized string. The following fields are part of the payload:
- `range`: The range in the spreadsheet (e.g., `A1`, `B2:D5`).
- `major_dimension`: Specifies how the data is organized (e.g., `ROWS`, `COLUMNS`).
- `values`: The data to be inserted into the sheet.
  
#### Usage Example
This is an example of a connector configuration file:

```yaml
# config-example.yaml
apiVersion: 0.1.0
meta:
  version: 0.1.0
  name: my-sheet-connector
  type: sheet-connector-sink
  topic: sheet-connector-topic
  create-topic: true
secrets:
  - name: GOOGLE_PRIVATE_KEY
  - name: GOOGLE_CLIENT_EMAIL
  - name: GOOGLE_TOKEN_URI  
sheet:
  google_private_key: ${{ secrets.GOOGLE_PRIVATE_KEY }}
  google_client_email: ${{ secrets.GOOGLE_CLIENT_EMAIL }}
  google_token_url: ${{ secrets.GOOGLE_TOKEN_URI }}
```

#### Setup Secrets file
The file which has secrets like `google_private_key`,`google_client_email`
`google_token_url`
```txt
GOOGLE_PRIVATE_KEY="-----BEGIN PRIVATE KEY-----private-key----END PRIVATE KEY-----\n"
GOOGLE_CLIENT_EMAIL="example.iam.gserviceaccount.com"
GOOGLE_TOKEN_URI="https://oauth2.googleapis.com/token"
```


#### Sample Payload
The payload represents the data to be written to the Google Sheets. Here’s a sample payload format:

```json
{
  "range": "A1",
  "major_dimension": "ROWS",
  "values": [["A1 value", "A2 value", "A3 value"]],
  "spreadsheet_id": "spreadsheet_id"
}
```

#### Running the Connector
1. **Set up the Connector**:
   Define the `config-example.yaml` file as shown above, providing the necessary secrets and spreadsheet ID.
   
2. **Deploy the Connector**:
   You can run the Google Sheets sink connector using the following command:
   
   ```bash
   cdk deploy start --config config-example.yaml  --secrets secrets.txt
   ```

3. **Send Data to Fluvio**:
   Produce records to the Fluvio topic defined in the config (e.g., `sheet-connector-topic`):

   ```bash
   fluvio topic produce sheet-connector-topic
   {"range": "A1", "major_dimension": "ROWS", "values": [["A1 value", "A2 value", "A3 value"]]}
   ```

4. **Verify Data in Google Sheets**:
   Open the Google Spreadsheet specified by the `spreadsheet_id` in the config and check if the data has been written successfully.


#### Google SpreadSheet Permission Error
If you get error like:
```bash
Bad Request: {"error":{"code":403,"message":"The caller does not have permission","status":"PERMISSION_DENIED"}}
```
Fix by adding `google_client_email` in Google SpreadSheet editor list. see 
[Fix On StackOverflow](https://stackoverflow.com/questions/38949318/google-sheets-api-returns-the-caller-does-not-have-permission-when-using-serve)  

#### Debugging
View the logs of the connector to troubleshoot any issues:
```bash
cdk deploy log my-sheet-connector
```

--- 
