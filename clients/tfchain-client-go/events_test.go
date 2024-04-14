package substrate

import (
	"fmt"
	"os"
	"reflect"
	"strings"
	"testing"

	"github.com/centrifuge/go-substrate-rpc-client/v4/types"
	"github.com/stretchr/testify/require"
)

func TestEventsTypes(t *testing.T) {
	require := require.New(t)

	var mgr Manager
	if _, ok := os.LookupEnv("CI"); ok {
		mgr = NewManager("ws://127.0.0.1:9944")
	} else {
		mgr = NewManager("wss://tfchain.dev.grid.tf")
	}

	con, meta, err := mgr.Raw()

	require.NoError(err)
	defer con.Client.Close()

	//fmt.Println(meta.Version)
	require.EqualValues(14, meta.Version)
	data := meta.AsMetadataV14

	var known EventRecords
	knownType := reflect.TypeOf(known)
	//data.FindEventNamesForEventID(eventID types.EventID)

	for _, mod := range data.Pallets {
		if !mod.HasEvents {
			continue
		}

		typ, ok := data.EfficientLookup[mod.Events.Type.Int64()]
		if !ok {
			continue
		}
		for _, variant := range typ.Def.Variant.Variants {
			name := fmt.Sprintf("%s_%s", mod.Name, variant.Name)
			filed, ok := knownType.FieldByName(name)
			if !ok {
				t.Fatalf("event %s not defined in known events", name)
				continue
			}
			//fmt.Println(" - Event: ", variant.Name)
			t.Run(name, func(t *testing.T) {
				eventValidator(t, &data, name, filed, variant)
			})
		}
	}
}

func eventValidator(t *testing.T, data *types.MetadataV14, name string, local reflect.StructField, remote types.Si1Variant) {
	require := require.New(t)
	//first of all, each local filed should be an SliceOf(remote) type.
	//which means
	require.True(local.Type.Kind() == reflect.Slice, "found: %+v", local.Type.Kind())
	elem := local.Type.Elem()
	// each element in that array is itself a structure, so we also must do this
	require.True(elem.Kind() == reflect.Struct, "found: %+v", elem.Kind())

	// each local type must start with the field:
	// `Phase    types.Phase`
	// and ends with the field
	// 	`Topics   []types.Hash`
	// which means number of local fields is always (remote + 2) fields

	require.EqualValuesf(elem.NumField()-2, len(remote.Fields),
		"type '%s' has %d fields while remote has '%d'", name, elem.NumField()-2, len(remote.Fields))

	first := elem.Field(0)
	last := elem.Field(elem.NumField() - 1)

	require.True(first.Type == reflect.TypeOf(types.Phase{}), "local type is missing phase field")
	require.True(last.Type == reflect.TypeOf([]types.Hash{}), "local type is missing topics field")
	// now we just need to validate everything in between
	for i := 1; i <= elem.NumField()-2; i++ {
		localFiled := elem.Field(i)

		fieldType, ok := data.EfficientLookup[remote.Fields[i-1].Type.Int64()]
		t.Run(fmt.Sprintf("%s(%d)", localFiled.Name, remote.Fields[i-1].Type.Int64()), func(t *testing.T) {
			require.True(ok, "couldn't lookup type")
			fieldValidator(t, data, localFiled.Type, fieldType)
		})

	}
}

// validate the inner type of an array or a slice with local type
func sliceInnerValidator(t *testing.T, data *types.MetadataV14, local reflect.Type, inner *types.Si1Type) {
	require := require.New(t)
	if inner.Def.IsPrimitive && inner.Def.Primitive.Si0TypeDefPrimitive == types.IsU8 {
		// a slice of a primitive type.
		// there is a special case for strings we need to handle separately
		if local.Kind() != reflect.String && local != reflect.TypeOf([]uint8{}) {
			t.Errorf("expecting a string or a []uin8 found: %+v", inner)
		}
		return
	}

	// slice of a more complex type.
	require.Equal(reflect.Slice, local.Kind(), "expecting a slice instead found %s", local.Kind())
	t.Run("T", func(t *testing.T) {
		fieldValidator(t, data, local.Elem(), inner)
	})
}

func fieldValidator(t *testing.T, data *types.MetadataV14, local reflect.Type, remote *types.Si1Type) {
	require := require.New(t)
	if remote.Def.IsPrimitive {
		prim := remote.Def.Primitive
		localKind := local.Kind()
		switch prim.Si0TypeDefPrimitive {
		case types.IsBool:
			require.EqualValues(reflect.Bool, localKind, "local field of the wrong type: %s expected: %d", localKind, prim.Si0TypeDefPrimitive)
		case types.IsU8:
			require.EqualValues(reflect.Uint8, localKind, "local field of the wrong type: %s expected: %d", localKind, prim.Si0TypeDefPrimitive)
		case types.IsU16:
			require.EqualValues(reflect.Uint16, localKind, "local field of the wrong type: %s expected: %d", localKind, prim.Si0TypeDefPrimitive)
		case types.IsU32:
			// we have exact match
			if localKind != reflect.Uint32 && local != reflect.TypeOf(Versioned{}) {
				t.Fatalf("local filed of wrong type: %s expected U32 or Versioned", localKind)
			}
		case types.IsU64:
			require.EqualValues(reflect.Uint64, localKind, "local field of the wrong type: %s expected: %d", localKind, prim.Si0TypeDefPrimitive)
		case types.IsU128:
			require.EqualValues(reflect.TypeOf(types.U128{}), local)
		default:
			// right now those are enough to match all primitive types defined in all the events types
			// but we might need to extend it in the future
			t.Fatalf("unknown primitive type '%v' (corresponding local filed is of type: %s)", remote, localKind)
		}
		return
	} else if remote.Def.IsComposite {
		composite := remote.Def.Composite

		// this can be a vector, so we need to check that
		if inner, isSlice := asSlice(data, composite); isSlice {
			t.Run("[T]", func(t *testing.T) {
				sliceInnerValidator(t, data, local, inner)
			})
			return
		}

		// handle some known types like AccountID and Hash
		pathEnd := remote.Path[len(remote.Path)-1]
		if pathEnd == "AccountId32" && (local == reflect.TypeOf(AccountID{}) || local == reflect.TypeOf(types.AccountID{})) {
			return
		} else if pathEnd == "H256" && (local == reflect.TypeOf(types.Hash{})) {
			return
		} else if pathEnd == "Public" && local.Kind() == reflect.Array {
			return
		} else if pathEnd == "Weight" && (local == reflect.TypeOf(types.Weight{})) {
			return
		}

		// not a slice. then we can compare field by field again
		require.Equal(reflect.Struct, local.Kind(), "expected '%+v' found '%s'", remote, local.Kind())
		for i, field := range composite.Fields {
			fieldRemoteType, ok := data.EfficientLookup[field.Type.Int64()]
			require.True(ok, "type with id '%d' not found", field.Type.Int64())

			fieldLocalType := local.Field(i)

			t.Run(string(field.Name), func(t *testing.T) {
				fieldValidator(t, data, fieldLocalType.Type, fieldRemoteType)
			})
		}

		return
	} else if remote.Def.IsSequence {
		inner := data.EfficientLookup[remote.Def.Sequence.Type.Int64()]
		t.Run("[T]", func(t *testing.T) {
			sliceInnerValidator(t, data, local, inner)
		})
		return
		//inner, ok := asSlice(data, composite types.Si1TypeDefComposite)
	} else if remote.Def.IsTuple {
		tuple := remote.Def.Tuple
		require.Equal(reflect.Struct, local.Kind(), "expected struct found: '%s'", local.Kind())
		require.Equal(len(tuple), local.NumField(), "expected tuple fields count to match local")
		for i, remoteField := range tuple {
			remoteFieldType := data.EfficientLookup[remoteField.Int64()]
			localFieldType := local.Field(i)

			t.Run(fmt.Sprint(i), func(t *testing.T) {
				fieldValidator(t, data, localFieldType.Type, remoteFieldType)
			})
		}
		return
	} else if remote.Def.IsVariant {
		// that's for enum. right now I can't find a way to validate this because
		// we can't look up the corresponding varian name in the go structure because
		// it's completely custom decoder/encoder. hence there is no automated mapping
		t.Skipf("enum types are not supported: %s", local.Name())
		return
	}

	// last thing we wanna check is
	t.Errorf("unknown field type: %s", defToString(&remote.Def))
}

func defToString(def *types.Si1TypeDef) string {
	if def.IsArray {
		return "Array"
	} else if def.IsBitSequence {
		return "BitSequence"
	} else if def.IsCompact {
		return "Compact"
	} else if def.IsComposite {
		return "Composite"
	} else if def.IsHistoricMetaCompat {
		return "HistoricMetaCompat"
	} else if def.IsPrimitive {
		return "Primitive"
	} else if def.IsSequence {
		return "Sequence"
	} else if def.IsTuple {
		return "Tuple"
	} else if def.IsVariant {
		return "Variant"
	} else {
		return "Unknown Type"
	}
}

func asSlice(data *types.MetadataV14, composite types.Si1TypeDefComposite) (inner *types.Si1Type, ok bool) {
	if len(composite.Fields) > 1 || !composite.Fields[0].HasTypeName {
		return nil, false
	}
	// that can be a special type of structures, like Vec or BoundedVec
	// these can locally map to slice, or string

	single := composite.Fields[0]
	typeName := string(single.TypeName)

	if strings.HasPrefix(typeName, "BoundedVec<") {
		// get inner type
		inner := data.EfficientLookup[single.Type.Int64()]
		if !inner.Def.IsComposite {
			return nil, false
		}

		return asSlice(data, inner.Def.Composite)
	} else if strings.HasPrefix(typeName, "Vec<") {
		inner := data.EfficientLookup[single.Type.Int64()]
		// the inner type can itself be a sequence for some reason :shrug:
		if inner.Def.IsSequence {
			return data.EfficientLookup[inner.Def.Sequence.Type.Int64()], true
		}
		return inner, true
	}

	return nil, false
}
