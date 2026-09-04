#ifndef NORMORDIS_PDF_H
#define NORMORDIS_PDF_H

#include <stddef.h>

#ifdef __cplusplus
extern "C" {
#endif

// Estrutura para retornar dados do PDF
typedef struct {
    unsigned char* data;
    size_t len;
} PdfResult;

// Gera um PDF a partir de um JSON de configuração
PdfResult* generate_pdf_from_json(const char* json_config);

// Gera um PDF a partir de um template NDT (JSON) e dados de runtime (JSON)
PdfResult* generate_pdf_from_ndt(const char* ndt_json, const char* data_json);

// Libera a memória alocada por generate_pdf_from_json / generate_pdf_from_ndt
void free_pdf_result(PdfResult* result);

#ifdef __cplusplus
}
#endif

#endif // NORMORDIS_PDF_H