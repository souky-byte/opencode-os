# Message

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**id** | **String** |  | 
**session_id** | **String** |  | 
**role** | **String** |  | 
**time** | [**models::AssistantMessageTime**](AssistantMessage_time.md) |  | 
**summary** | Option<**bool**> |  | [optional]
**agent** | **String** |  | 
**model** | [**models::SessionPromptRequestModel**](session_prompt_request_model.md) |  | 
**system** | Option<**String**> |  | [optional]
**tools** | Option<**std::collections::HashMap<String, bool>**> |  | [optional]
**variant** | Option<**String**> |  | [optional]
**error** | Option<[**models::AssistantMessageError**](AssistantMessage_error.md)> |  | [optional]
**parent_id** | **String** |  | 
**model_id** | **String** |  | 
**provider_id** | **String** |  | 
**mode** | **String** |  | 
**path** | [**models::AssistantMessagePath**](AssistantMessage_path.md) |  | 
**cost** | **f64** |  | 
**tokens** | [**models::AssistantMessageTokens**](AssistantMessage_tokens.md) |  | 
**finish** | Option<**String**> |  | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


