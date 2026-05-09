package main

/*
#cgo CFLAGS: -I/path/to/normordis-pdf
#cgo LDFLAGS: -L/path/to/normordis-pdf/target/release -lnormordis_pdf
#include "normordis_pdf.h"
#include <stdlib.h>
*/
import "C"
import (
	"fmt"
	"io/ioutil"
	"unsafe"
)

func main() {
	// Exemplo de JSON para gerar um PDF simples
	jsonConfig := C.CString(`{
		"title": "Documento Teste",
		"elements": [
			{"type": "paragraph", "text": "Olá do Go via normordis-pdf!", "align": "left"}
		]
	}`)

	// Chama a função FFI
	result := C.generate_pdf_from_json(jsonConfig)

	if result == nil {
		fmt.Println("Erro ao gerar PDF")
		C.free(unsafe.Pointer(jsonConfig))
		return
	}

	// Converte os bytes para Go
	pdfBytes := C.GoBytes(unsafe.Pointer(result.data), C.int(result.len))

	// Salva o PDF
	err := ioutil.WriteFile("output.pdf", pdfBytes, 0644)
	if err != nil {
		fmt.Println("Erro ao salvar PDF:", err)
	}

	// Libera memória
	C.free_pdf_result(result)
	C.free(unsafe.Pointer(jsonConfig))

	fmt.Println("PDF gerado com sucesso: output.pdf")
}