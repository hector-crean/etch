# \StylesApi

All URIs are relative to *https://api.figma.com*

Method | HTTP request | Description
------------- | ------------- | -------------
[**get_style**](StylesApi.md#get_style) | **GET** /v1/styles/{key} | Get style



## get_style

> models::InlineObject20 get_style(key)
Get style

Get metadata on a style by key.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**key** | **String** | The unique identifier of the style. | [required] |

### Return type

[**models::InlineObject20**](inline_object_20.md)

### Authorization

[OAuth2](../README.md#OAuth2), [PersonalAccessToken](../README.md#PersonalAccessToken)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

