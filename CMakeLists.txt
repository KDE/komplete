# SPDX-License-Identifier: GPL-2.0-only OR GPL-3.0-only OR LicenseRef-KDE-Accepted-GPL
# SPDX-FileCopyrightText: 2026 Hadi Chokr <hadichokr@icloud.com>

cmake_minimum_required(VERSION 3.20)
project(komplete VERSION 1.0.0 LANGUAGES NONE)

include(GNUInstallDirs)

if(NOT CMAKE_INSTALL_PREFIX STREQUAL "/usr")
    message(WARNING
        "CMAKE_INSTALL_PREFIX is '${CMAKE_INSTALL_PREFIX}', but the komplete "
        "runtime expects to live under /usr. Shims and config lookup use "
        "absolute /usr paths. Use DESTDIR for staged installs.")
endif()

install(PROGRAMS
    src/komplete-install
    src/komplete-run
    DESTINATION lib/komplete
)

# Project trees
set(KOMPLETE_TREES "all" CACHE STRING
    "Semicolon-separated list of trees to install, or 'all'")


file(GLOB _tree_candidates RELATIVE ${CMAKE_CURRENT_SOURCE_DIR}/trees
    CONFIGURE_DEPENDS ${CMAKE_CURRENT_SOURCE_DIR}/trees/*)

set(_available "")
foreach(_entry IN LISTS _tree_candidates)
    if(IS_DIRECTORY ${CMAKE_CURRENT_SOURCE_DIR}/trees/${_entry})
        list(APPEND _available ${_entry})
    endif()
endforeach()

if(KOMPLETE_TREES STREQUAL "all")
    set(_selected ${_available})
else()
    set(_selected ${KOMPLETE_TREES})
endif()

foreach(tree IN LISTS _selected)
    set(_tree_dir ${CMAKE_CURRENT_SOURCE_DIR}/trees/${tree})
    if(NOT IS_DIRECTORY ${_tree_dir}/usr)
        message(FATAL_ERROR
            "Tree '${tree}' does not exist or has no usr/ directory. "
            "Available trees: ${_available}")
    endif()
    message(STATUS "Installing komplete tree: ${tree}")
    # Trailing slash: copy the *contents* of usr/ into the prefix.
    install(DIRECTORY ${_tree_dir}/usr/
        DESTINATION .
        USE_SOURCE_PERMISSIONS
        PATTERN "*.license" EXCLUDE
    )
endforeach()

find_program(MSGFMT_EXECUTABLE msgfmt)
file(GLOB _po_files CONFIGURE_DEPENDS ${CMAKE_CURRENT_SOURCE_DIR}/po/*.po)

if(_po_files AND NOT MSGFMT_EXECUTABLE)
    message(WARNING "msgfmt not found; translations will not be built")
endif()

if(MSGFMT_EXECUTABLE AND _po_files)
    file(MAKE_DIRECTORY ${CMAKE_CURRENT_BINARY_DIR}/po)
    set(_mo_files "")
    foreach(_po IN LISTS _po_files)
        get_filename_component(_lang ${_po} NAME_WE)
        set(_mo ${CMAKE_CURRENT_BINARY_DIR}/po/${_lang}.mo)
        add_custom_command(OUTPUT ${_mo}
            COMMAND ${MSGFMT_EXECUTABLE} -o ${_mo} ${_po}
            DEPENDS ${_po}
            COMMENT "Generating ${_lang}.mo")
        list(APPEND _mo_files ${_mo})
        install(FILES ${_mo}
            DESTINATION ${CMAKE_INSTALL_LOCALEDIR}/${_lang}/LC_MESSAGES
            RENAME komplete.mo)
    endforeach()
    add_custom_target(translations ALL DEPENDS ${_mo_files})
endif()
