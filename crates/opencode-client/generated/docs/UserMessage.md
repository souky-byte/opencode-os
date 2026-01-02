# UserMessage

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**id** | **String** |  | 
**session_id** | **String** |  | 
**role** | **String** |  | 
**time** | [**models::UserMessageTime**](UserMessage_time.md) |  | 
**summary** | Option<[**models::UserMessageSummary**](UserMessage_summary.md)> |  | [optional]
**agent** | **String** |  | 
**model** | [**models::SessionPromptRequestModel**](session_prompt_request_model.md) |  | 
**system** | Option<**String**> |  | [optional]
**tools** | Option<**std::collections::HashMap<String, bool>**> |  | [optional]
**variant** | Option<**String**> |  | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


