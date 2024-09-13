package main

import (
	"encoding/base64"
	"encoding/binary"
	"encoding/json"
	"log"
	"math"
	"os"
	"strconv"

	"rusty-chunkenc-golang-tests/chunkenc"
)

type JVarbitInt struct {
	Value   int64  `json:"v"`
	Encoded []byte `json:"e"`
}

type JVarbitUint struct {
	Value   uint64 `json:"v"`
	Encoded []byte `json:"e"`
}

type JUvarint struct {
	Value   uint64 `json:"v"`
	Encoded []byte `json:"e"`
}

type JSample struct {
	Timestamp int64   `json:"ts"`
	Value     float64 `json:"v"`
}

type JChunk struct {
	Samples []JSample `json:"s"`
	Encoded []byte    `json:"e"`
}

type TestJson struct {
	VarbitInts  []JVarbitInt  `json:"varbit_ints"`
	VarbitUints []JVarbitUint `json:"varbit_uints"`
	Uvarints    []JUvarint    `json:"uvarints"`
	Chunks      []JChunk      `json:"chunks"`
}

func (i JVarbitInt) MarshalJSON() ([]byte, error) {
	type Alias JVarbitInt
	return json.Marshal(&struct {
		*Alias
		Encoded string `json:"e"`
	}{
		Alias:   (*Alias)(&i),
		Encoded: base64.RawStdEncoding.EncodeToString(i.Encoded),
	})
}

func (u JVarbitUint) MarshalJSON() ([]byte, error) {
	type Alias JVarbitUint
	return json.Marshal(&struct {
		*Alias
		Encoded string `json:"e"`
	}{
		Alias:   (*Alias)(&u),
		Encoded: base64.RawStdEncoding.EncodeToString(u.Encoded),
	})
}

func (u JUvarint) MarshalJSON() ([]byte, error) {
	type Alias JUvarint
	return json.Marshal(&struct {
		*Alias
		Encoded string `json:"e"`
	}{
		Alias:   (*Alias)(&u),
		Encoded: base64.RawStdEncoding.EncodeToString(u.Encoded),
	})
}

func make_varbitint_samples() []JVarbitInt {
	// Define some example numbers
	numbers := []int64{
		math.MinInt64,
		-36028797018963968, -36028797018963967,
		-16777216, -16777215,
		-131072, -131071,
		-2048, -2047,
		-256, -255,
		-32, -31,
		-4, -3,
		-1, 0, 1,
		4, 5,
		32, 33,
		256, 257,
		2048, 2049,
		131072, 131073,
		16777216, 16777217,
		36028797018963968, 36028797018963969,
		math.MaxInt64,
	}

	// Create an array of Varbit objects
	var varbitArray []JVarbitInt
	for _, val := range numbers {
		bs := chunkenc.Bstream{}
		chunkenc.PutVarbitInt(&bs, val)
		encoded := bs.Bytes()
		varbit := JVarbitInt{
			Value:   val,
			Encoded: encoded,
		}
		varbitArray = append(varbitArray, varbit)
	}

	return varbitArray
}

func make_varbituint_samples() []JVarbitUint {
	// Define some example numbers
	numbers := []uint64{
		0, 1,
		7, 8,
		63, 64,
		511, 512,
		4095, 4096,
		262143, 262144,
		33554431, 33554432,
		72057594037927935, 72057594037927936,
		math.MaxUint64,
	}

	// Create an array of Uvarint objects
	var uvarintArray []JVarbitUint
	for _, val := range numbers {
		bs := chunkenc.Bstream{}
		chunkenc.PutVarbitUint(&bs, val)
		encoded := bs.Bytes()
		uvarint := JVarbitUint{
			Value:   val,
			Encoded: encoded,
		}
		uvarintArray = append(uvarintArray, uvarint)
	}

	return uvarintArray
}

func make_uvarint_samples() []JUvarint {
	// Define some example numbers
	numbers := []uint64{
		0, 1,
		7, 8,
		63, 64,
		511, 512,
		4095, 4096,
		262143, 262144,
		33554431, 33554432,
		72057594037927935, 72057594037927936,
		math.MaxUint64,
	}

	// Create an array of Uvarint objects
	var uvarintArray []JUvarint
	for _, val := range numbers {
		buf := make([]byte, binary.MaxVarintLen64)
		n := binary.PutUvarint(buf, val)
		uvarint := JUvarint{
			Value:   val,
			Encoded: buf[:n],
		}
		uvarintArray = append(uvarintArray, uvarint)
	}

	return uvarintArray
}

func make_chunk_samples(n int) []JChunk {
	start_ts := int64(1234123324)
	start_val := 123535.123

	samples := make([]JSample, 0)
	for i := 0; i < n; i++ {
		ts := start_ts + int64(i)
		val := start_val + float64(i)
		samples = append(samples, JSample{ts, val})
	}

	chunks := make([]JChunk, 0)

	chunk := chunkenc.NewXORChunk()
	app, err := chunk.Appender()
	if err != nil {
		log.Fatalf("Failed to create appender: %v", err)
	}
	for _, sample := range samples {
		app.Append(sample.Timestamp, sample.Value)
	}

	bytes := chunk.Bytes()

	chunks = append(chunks, JChunk{
		samples,
		bytes,
	})

	return chunks
}

func main() {
	// read n as first argument
	n, err := strconv.Atoi(os.Args[1])
	if err != nil {
		log.Fatalf("Failed to parse n: %v", err)
	}

	//n := 1399

	samples := TestJson{
		VarbitInts:  make_varbitint_samples(),
		VarbitUints: make_varbituint_samples(),
		Uvarints:    make_uvarint_samples(),
		Chunks:      make_chunk_samples(n),
	}

	_, err = json.MarshalIndent(samples, "", "  ")
	if err != nil {
		log.Fatalf("Failed to marshal JSON: %v", err)
	}

	//fmt.Println(string(jsonData))
}
