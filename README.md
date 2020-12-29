Huge Page Support of Theseus - CPSC626

Namitha Liyanage, Bowen Huang

# Link to the source codes

https://github.com/NamiLiy/Theseus/commits/cpsc626_unified 
https://github.com/NamiLiy/Theseus/tree/cpsc626

# Percentage of the credits

Namitha Liyanage :  50%
Bowen Huang :  50%

# How to run

Test application : applications/huge_page_test

This application creates pages of 4KB, 2MB and 1GB depending on architectural support.

# Description

To implement huge page support within the theseus, basically we need to write logics that can allocate huge pages and map those allocated hugepages to physical frames. 

Within the Theseus, there is a clear path of performing allocation & mapping.
create_mapping is the entry point that we exposed to user program, and it’s PageSize-ignostic. Create_mapping further calls allocate_pages_by_bytes and map_allocated_pages to perform actual allocation and mapping. 

# Improvement over our previous submission

Unified Implementation

Our previous implementation just duplicated every single function or data structure related with page allocation & mapping, which means that our previous implementation was actually separated from the rest of Theseus OS. 

This new submission offers a unified implementation, we used the same underlying data structure and basic framework of logic, and embedded our implementation within it. We modified Page to hold an extra variable called page_size to effectively track the size of page when used in AllocatedPages and MappedPages. This version is available at https://github.com/NamiLiy/Theseus/commits/cpsc626_unified

Language Features

We then extended part a by converting page_size to a trait called PageType and Converting Page, AllocatedPages and MappedPages to generics named Page<PageType>, AllocatedPages<PageType> and MappedPages<PageType>.
This version is incomplete as not all the places MappedPages are being used are not substituted with generics. Available at https://github.com/NamiLiy/Theseus/tree/cpsc626

