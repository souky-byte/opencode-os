# Part

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**id** | **String** |  | 
**session_id** | **String** |  | 
**message_id** | **String** |  | 
**r#type** | **String** |  | 
**text** | **String** |  | 
**synthetic** | Option<**bool**> |  | [optional]
**ignored** | Option<**bool**> |  | [optional]
**time** | [**models::UserMessageTime**](UserMessage_time.md) |  | 
**metadata** | Option<[**std::collections::HashMap<String, serde_json::Value>**](serde_json::Value.md)> |  | [optional]
**prompt** | **String** |  | 
**description** | **String** |  | 
**agent** | **String** |  | 
**command** | Option<**String**> |  | [optional]
**mime** | **String** |  | 
**filename** | Option<**String**> |  | [optional]
**url** | **String** |  | 
**source** | Option<[**models::AgentPartSource**](AgentPart_source.md)> |  | [optional]
**call_id** | **String** |  | 
**tool** | **String** |  | 
**state** | [**models::ToolState**](ToolState.md) |  | 
**snapshot** | **String** |  | 
**reason** | **String** |  | 
**cost** | **f64** |  | 
**tokens** | [**models::AssistantMessageTokens**](AssistantMessage_tokens.md) |  | 
**hash** | **String** |  | 
**files** | **Vec<String>** |  | 
**name** | **String** |  | 
**attempt** | **f64** |  | 
**error** | [**models::ApiError**](APIError.md) |  | 
**auto** | **bool** |  | 

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


