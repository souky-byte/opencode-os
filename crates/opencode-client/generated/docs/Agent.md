# Agent

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**name** | **String** |  | 
**description** | Option<**String**> |  | [optional]
**mode** | **String** |  | 
**native** | Option<**bool**> |  | [optional]
**hidden** | Option<**bool**> |  | [optional]
**default** | Option<**bool**> |  | [optional]
**top_p** | Option<**f64**> |  | [optional]
**temperature** | Option<**f64**> |  | [optional]
**color** | Option<**String**> |  | [optional]
**permission** | [**models::AgentPermission**](Agent_permission.md) |  | 
**model** | Option<[**models::SessionPromptRequestModel**](session_prompt_request_model.md)> |  | [optional]
**prompt** | Option<**String**> |  | [optional]
**tools** | **std::collections::HashMap<String, bool>** |  | 
**options** | [**std::collections::HashMap<String, serde_json::Value>**](serde_json::Value.md) |  | 
**max_steps** | Option<**i32**> |  | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


