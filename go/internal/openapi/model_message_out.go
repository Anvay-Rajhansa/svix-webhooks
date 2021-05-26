/*
 * Svix
 *
 * The Svix server API documentation
 *
 * API version: 1.4
 */

// Code generated by OpenAPI Generator (https://openapi-generator.tech); DO NOT EDIT.

package openapi

import (
	"encoding/json"
	"time"
)

// MessageOut struct for MessageOut
type MessageOut struct {
	EventType string `json:"eventType"`
	EventId *string `json:"eventId,omitempty"`
	Data map[string]interface{} `json:"data"`
	Id string `json:"id"`
	Timestamp time.Time `json:"timestamp"`
}

// NewMessageOut instantiates a new MessageOut object
// This constructor will assign default values to properties that have it defined,
// and makes sure properties required by API are set, but the set of arguments
// will change when the set of required properties is changed
func NewMessageOut(eventType string, data map[string]interface{}, id string, timestamp time.Time, ) *MessageOut {
	this := MessageOut{}
	this.EventType = eventType
	this.Data = data
	this.Id = id
	this.Timestamp = timestamp
	return &this
}

// NewMessageOutWithDefaults instantiates a new MessageOut object
// This constructor will only assign default values to properties that have it defined,
// but it doesn't guarantee that properties required by API are set
func NewMessageOutWithDefaults() *MessageOut {
	this := MessageOut{}
	return &this
}

// GetEventType returns the EventType field value
func (o *MessageOut) GetEventType() string {
	if o == nil  {
		var ret string
		return ret
	}

	return o.EventType
}

// GetEventTypeOk returns a tuple with the EventType field value
// and a boolean to check if the value has been set.
func (o *MessageOut) GetEventTypeOk() (*string, bool) {
	if o == nil  {
		return nil, false
	}
	return &o.EventType, true
}

// SetEventType sets field value
func (o *MessageOut) SetEventType(v string) {
	o.EventType = v
}

// GetEventId returns the EventId field value if set, zero value otherwise.
func (o *MessageOut) GetEventId() string {
	if o == nil || o.EventId == nil {
		var ret string
		return ret
	}
	return *o.EventId
}

// GetEventIdOk returns a tuple with the EventId field value if set, nil otherwise
// and a boolean to check if the value has been set.
func (o *MessageOut) GetEventIdOk() (*string, bool) {
	if o == nil || o.EventId == nil {
		return nil, false
	}
	return o.EventId, true
}

// HasEventId returns a boolean if a field has been set.
func (o *MessageOut) HasEventId() bool {
	if o != nil && o.EventId != nil {
		return true
	}

	return false
}

// SetEventId gets a reference to the given string and assigns it to the EventId field.
func (o *MessageOut) SetEventId(v string) {
	o.EventId = &v
}

// GetData returns the Data field value
func (o *MessageOut) GetData() map[string]interface{} {
	if o == nil  {
		var ret map[string]interface{}
		return ret
	}

	return o.Data
}

// GetDataOk returns a tuple with the Data field value
// and a boolean to check if the value has been set.
func (o *MessageOut) GetDataOk() (*map[string]interface{}, bool) {
	if o == nil  {
		return nil, false
	}
	return &o.Data, true
}

// SetData sets field value
func (o *MessageOut) SetData(v map[string]interface{}) {
	o.Data = v
}

// GetId returns the Id field value
func (o *MessageOut) GetId() string {
	if o == nil  {
		var ret string
		return ret
	}

	return o.Id
}

// GetIdOk returns a tuple with the Id field value
// and a boolean to check if the value has been set.
func (o *MessageOut) GetIdOk() (*string, bool) {
	if o == nil  {
		return nil, false
	}
	return &o.Id, true
}

// SetId sets field value
func (o *MessageOut) SetId(v string) {
	o.Id = v
}

// GetTimestamp returns the Timestamp field value
func (o *MessageOut) GetTimestamp() time.Time {
	if o == nil  {
		var ret time.Time
		return ret
	}

	return o.Timestamp
}

// GetTimestampOk returns a tuple with the Timestamp field value
// and a boolean to check if the value has been set.
func (o *MessageOut) GetTimestampOk() (*time.Time, bool) {
	if o == nil  {
		return nil, false
	}
	return &o.Timestamp, true
}

// SetTimestamp sets field value
func (o *MessageOut) SetTimestamp(v time.Time) {
	o.Timestamp = v
}

func (o MessageOut) MarshalJSON() ([]byte, error) {
	toSerialize := map[string]interface{}{}
	if true {
		toSerialize["eventType"] = o.EventType
	}
	if o.EventId != nil {
		toSerialize["eventId"] = o.EventId
	}
	if true {
		toSerialize["data"] = o.Data
	}
	if true {
		toSerialize["id"] = o.Id
	}
	if true {
		toSerialize["timestamp"] = o.Timestamp
	}
	return json.Marshal(toSerialize)
}

type NullableMessageOut struct {
	value *MessageOut
	isSet bool
}

func (v NullableMessageOut) Get() *MessageOut {
	return v.value
}

func (v *NullableMessageOut) Set(val *MessageOut) {
	v.value = val
	v.isSet = true
}

func (v NullableMessageOut) IsSet() bool {
	return v.isSet
}

func (v *NullableMessageOut) Unset() {
	v.value = nil
	v.isSet = false
}

func NewNullableMessageOut(val *MessageOut) *NullableMessageOut {
	return &NullableMessageOut{value: val, isSet: true}
}

func (v NullableMessageOut) MarshalJSON() ([]byte, error) {
	return json.Marshal(v.value)
}

func (v *NullableMessageOut) UnmarshalJSON(src []byte) error {
	v.isSet = true
	return json.Unmarshal(src, &v.value)
}


