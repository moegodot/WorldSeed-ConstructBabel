#pragma once

#include <plutosvg/plutosvg-ft.h>

#include <ft2build.h>

#include FT_FREETYPE_H
#include FT_MODULE_H

extern "C"
{
    int bind_plutosvg_freetype(FT_Library library);
}
