#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals, dead_code)]

use Language_Agnostic_Formatting_Style as lafs;
use lafs:: { Config_Settings, Source_Formatter, Source_Language };

#[test]
fn Test_Batch_User_Kotlin_Scalac_Format()
{
    let Input = r#"@echo off

rem Based on scalac.bat from the Scala distribution
rem Copyright 2002-2011, LAMP/EPFL
rem Copyright 2010-2026 JetBrains s.r.o. and Kotlin Programming Language contributors.
rem Use of this source code is governed by the Apache 2.0 license that can be found in the license/LICENSE.txt file.

rem We adopt the following conventions:
rem - System/user environment variables start with a letter
rem - Local batch variables start with an underscore ('_')

setlocal
call :set_home

if "%_KOTLIN_COMPILER%"=="" set _KOTLIN_COMPILER=org.jetbrains.kotlin.cli.jvm.K2JVMCompiler

rem Prepend JAVA_HOME to local PATH to be able to simply execute "java" later in the script.
if not "%JAVA_HOME%"=="" set "PATH=%JAVA_HOME%\bin;%PATH%"

rem We use the value of the JAVA_OPTS environment variable if defined
if "%JAVA_OPTS%"=="" set JAVA_OPTS=-Xmx512M -Xms128M

rem Iterate through arguments and split them into java and kotlin ones
:loop
set _arg=%~1
if "%_arg%"=="" goto loopend

if "%_arg:~0,2%"=="-J" (
                           if "%_arg:~2%"=="" ( echo error: empty -J argument & goto error )
                           set JAVA_OPTS=%JAVA_OPTS% "%_arg:~2%"
                       ) ^
else (
         if "%_arg:~0,2%"=="-D" ( set JAVA_OPTS=%JAVA_OPTS% "%_arg%" ) else ( set KOTLIN_OPTS=%KOTLIN_OPTS% "%_arg%" )
     )
shift
goto loop
:loopend

setlocal EnableDelayedExpansion

call :set_java_version

rem Allow JNI access for all compiler code (jansi) and suppress unsafe deprecation warnings (KT-76799, IDEA-370928).
if !_java_major_version! geq 24 (
                                    set JAVA_OPTS=!JAVA_OPTS! "--enable-native-access=ALL-UNNAMED"
                                    set JAVA_OPTS=!JAVA_OPTS! "--sun-misc-unsafe-memory-access=allow"
                                )

if "!_KOTLIN_RUNNER!"=="1" (
                               java !JAVA_OPTS! "-Dkotlin.home=%_KOTLIN_HOME%" -cp "%_KOTLIN_HOME%\lib\kotlin-runner.jar" ^
                                    org.jetbrains.kotlin.runner.Main %KOTLIN_OPTS%
                           ) ^
else (
         set _ADDITIONAL_CLASSPATH=

         if !_java_major_version! lss 13 set JAVA_OPTS=!JAVA_OPTS! "-noverify"
         if not "%_KOTLIN_TOOL%"=="" set _ADDITIONAL_CLASSPATH=;%_KOTLIN_HOME%\lib\%_KOTLIN_TOOL%

         java !JAVA_OPTS! -cp "%_KOTLIN_HOME%\lib\kotlin-preloader.jar" ^
              org.jetbrains.kotlin.preloading.Preloader -cp "%_KOTLIN_HOME%\lib\kotlin-compiler.jar!_ADDITIONAL_CLASSPATH!" ^
              %_KOTLIN_COMPILER% %KOTLIN_OPTS%
     )

goto end

rem ##########################################################################
rem # subroutines

:set_home
set "_KOTLIN_HOME=%~dp0.."
goto :eof

rem Parses "java -version" output and stores the major version to _java_major_version.
rem Note that this only loads the first component of the version, so "1.8.0_265" -> "1".
rem But it's fine because major version is 9 for JDK 9, and so on.
rem Needs to be executed in the EnableDelayedExpansion mode.
:set_java_version
set _version=
rem Parse output and take the third token from the string containing " version ".
for /f "tokens=3" %%i in ('java -version 2^>^&1 ^| findstr /i " version "') do (
                                                                                  for /f "delims=-. tokens=1" %%j in ("%%i") do ( set _version=%%j )
                                                                               )
if "!_version!"=="" ( set _java_major_version=1 ) else ( set _java_major_version=!_version:~1! )
goto :eof

:error
set ERRORLEVEL=1

:end
exit /b %ERRORLEVEL%
"#;

    let mut Config = Config_Settings:: default();
    Config.Language = Source_Language::Batch;

    let Formatted = Source_Formatter::Format(Input, & Config).expect("Formatting failed");

    // Assert LF line endings only
    assert!( ! Formatted.contains("\r\n"),"Batch formatter must emit LF line endings, got CRLF");

    // Assert caret bridge on if-else closure
    assert!(Formatted.contains(") ^\nelse ("),"Expected ') ^\\nelse (' in formatted output:\n{}", Formatted);

    // Assert delimiter column alignment for if and else
    assert!(Formatted.contains("if \"%_arg:~0,2%\"==\"-J\" ("),"Expected header");
    assert!(Formatted.contains("                       ) ^"),"Expected closing paren aligned at col 23:\n{}", Formatted);
    assert!(Formatted.contains("else (\n         "),"Expected else child indented at col 9:\n{}", Formatted);
    assert!(Formatted.contains("     )\nshift"),"Expected closing paren of else at col 5:\n{}", Formatted);

    // Assert Monolith formatting for short inlines
    assert!(Formatted.contains("if \"!_version!\"==\"\" ( set _java_major_version=1 ) else ( set _java_major_version=!_version:~1! )"));

    // Assert idempotence
    let SecondPass = Source_Formatter::Format( & Formatted, & Config).expect("Second pass failed");
    assert_eq!(Formatted, SecondPass, "Batch formatting must be 100% idempotent!");
}

#[test]
fn Test_Batch_Idempotence()
{
    let Input = r#"if 1==1 (
            echo yes
        ) ^
else (
         echo no
     )
"#;
    let mut Config = Config_Settings:: default();
    Config.Language = Source_Language::Batch;

    let FirstPass = Source_Formatter::Format(Input, & Config).expect("First pass failed");
    let SecondPass = Source_Formatter::Format( & FirstPass, & Config).expect("Second pass failed");
    assert_eq!(FirstPass, SecondPass, "Must be idempotent");
}

#[test]
fn Test_Batch_For_Loop_Alignment()
{
    let Input = r#"for /f "tokens=3" %%i in ('java -version') do (
    for /f "tokens=1" %%j in ("%%i") do ( set V=%%j )
)
"#;
    let mut Config = Config_Settings:: default();
    Config.Language = Source_Language::Batch;

    let Formatted = Source_Formatter::Format(Input, & Config).expect("Formatting failed");
    let ColOpen = "for /f \"tokens=3\" %%i in ('java -version') do".len() + 1;
    // 45
    let ExpectedClose = format!("{})", " ".repeat(ColOpen));
    assert!(Formatted.contains( & ExpectedClose),"Closing paren must align with opening paren at col {}:\n{}", ColOpen, Formatted);
}
